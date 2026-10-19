use crate::{Amount, State, util::invalid};
use pco::{
    ChunkConfig, DeltaSpec, ModeSpec,
    standalone::{simple_compress, simple_decompress_into},
};
use std::{fs::File, io::Result, os::unix::fs::FileExt};
use zerocopy::IntoBytes;
/// A retained checkpoint: both columns pco-compressed, appended to the retained data.
const PACKED: &[u8; 8] = b"STATEP04";
/// A tip's page directory: one page slot per `PAGE` origins, in a page file of immutable slots.
pub(crate) const PAGED: &[u8; 8] = b"STATEV05";
pub(crate) const HEADER: usize = 72;
/// Origins per tip page (4 KiB of amounts): a block rewrites only the pages it changed.
pub(crate) const PAGE: usize = 256;
pub(crate) const PAGE_BYTES: u64 = (PAGE * size_of::<Amount>()) as u64;
/// Restores read tip pages in spans of at most this many slots, across gaps of at most
/// `MAX_GAP` garbage slots.
const MAX_SPAN: u32 = 1024;
const MAX_GAP: u32 = 16;
pub(crate) const MAX_RECORD: u64 = 2 * 1024 * 1024 * 1024;
const MAX_LEN: usize = 100_000_000;

/// Magic, length, hash, versions, then a format word: the sats section length of a packed
/// checkpoint, the page file generation of a tip.
pub(crate) fn header(
    magic: &[u8; 8],
    state: &State,
    versions: (u64, u64),
    word: u64,
) -> [u8; HEADER] {
    let mut header = [0; HEADER];
    header[..8].copy_from_slice(magic);
    header[8..16].copy_from_slice(&(state.len() as u64).to_le_bytes());
    header[16..48].copy_from_slice(&state.hash());
    header[48..56].copy_from_slice(&versions.0.to_le_bytes());
    header[56..64].copy_from_slice(&versions.1.to_le_bytes());
    header[64..72].copy_from_slice(&word.to_le_bytes());
    header
}
fn word(header: &[u8; HEADER], at: usize) -> u64 {
    u64::from_le_bytes(header[at..at + 8].try_into().unwrap())
}
pub(crate) fn len(header: &[u8; HEADER]) -> Result<usize> {
    usize::try_from(word(header, 8))
        .ok()
        .filter(|&len| len <= MAX_LEN)
        .ok_or_else(|| invalid("snapshot exceeds height limit"))
}
pub(crate) fn generation(header: &[u8; HEADER]) -> u64 {
    word(header, 64)
}
pub(crate) fn hash(header: &[u8; HEADER]) -> [u8; 32] {
    header[16..48].try_into().unwrap()
}
pub(crate) fn versions(header: &[u8; HEADER]) -> (u64, u64) {
    (word(header, 48), word(header, 56))
}

/// Appends a retained checkpoint at `offset`, returning its size.
pub(crate) fn write_packed(
    file: &File,
    offset: u64,
    state: &State,
    versions: (u64, u64),
) -> Result<u64> {
    let config = ChunkConfig::default()
        .with_compression_level(3)
        .with_mode_spec(ModeSpec::Classic)
        .with_delta_spec(DeltaSpec::NoOp);
    // One column buffer: sats, then refilled with counts.
    let mut column: Vec<_> = state.amounts().iter().map(|v| v.sats).collect();
    let sats = simple_compress(&column, &config).map_err(|_| invalid("sats compression failed"))?;
    column.clear();
    column.extend(state.amounts().iter().map(|v| v.count));
    let counts =
        simple_compress(&column, &config).map_err(|_| invalid("counts compression failed"))?;
    let bytes = (HEADER + sats.len() + counts.len()) as u64;
    if bytes > MAX_RECORD {
        return Err(invalid("snapshot exceeds page limit"));
    }
    let header = header(PACKED, state, versions, sats.len() as u64);
    file.write_all_at(&header, offset)?;
    file.write_all_at(&sats, offset + HEADER as u64)?;
    file.write_all_at(&counts, offset + HEADER as u64 + sats.len() as u64)?;
    Ok(bytes)
}
pub(crate) fn write_amounts(file: &File, offset: u64, amounts: &[Amount]) -> Result<()> {
    #[cfg(target_endian = "little")]
    let bytes = amounts.as_bytes();
    #[cfg(target_endian = "big")]
    let bytes = &amounts.iter().flat_map(|v| v.encode()).collect::<Vec<_>>();
    file.write_all_at(bytes, offset)
}
fn read_amounts(file: &File, offset: u64, amounts: &mut [Amount]) -> Result<()> {
    file.read_exact_at(amounts.as_mut_bytes(), offset)?;
    #[cfg(target_endian = "big")]
    for v in amounts {
        v.sats = u64::from_le(v.sats);
        v.count = u64::from_le(v.count);
    }
    Ok(())
}
pub(crate) fn read_slots(dir: &File, len: usize) -> Result<Vec<u32>> {
    let mut bytes = vec![0; len.div_ceil(PAGE) * 4];
    dir.read_exact_at(&mut bytes, HEADER as u64)?;
    Ok(bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|&slot| u32::from_le_bytes(slot))
        .collect())
}

enum Body {
    /// A retained checkpoint's payload in the retained data.
    Packed {
        file: File,
        offset: u64,
        bytes: usize,
    },
    /// A tip's directory and the page file it points into, as opened together.
    Paged { dir: File, pages: File },
}
/// Opening a snapshot reads its identity; reconstruction reads its payload once.
pub(crate) struct Snapshot {
    header: [u8; HEADER],
    body: Body,
    pub len: usize,
}
impl Snapshot {
    pub fn open_packed(
        file: &File,
        offset: u64,
        bytes: usize,
        versions: (u64, u64),
    ) -> Result<Option<Self>> {
        if bytes < HEADER {
            return Err(invalid("truncated snapshot"));
        }
        let mut header = [0; HEADER];
        file.read_exact_at(&mut header, offset)?;
        if &header[..8] != PACKED {
            return Err(invalid("unsupported snapshot format"));
        }
        Self::new(
            header,
            versions,
            Body::Packed {
                file: file.try_clone()?,
                offset: offset + HEADER as u64,
                bytes: bytes - HEADER,
            },
        )
    }
    /// `header` is the directory's, read when it was opened.
    pub fn open_paged(
        header: [u8; HEADER],
        dir: &File,
        pages: &File,
        versions: (u64, u64),
    ) -> Result<Option<Self>> {
        let body = Body::Paged {
            dir: dir.try_clone()?,
            pages: pages.try_clone()?,
        };
        Self::new(header, versions, body)
    }
    fn new(header: [u8; HEADER], versions: (u64, u64), body: Body) -> Result<Option<Self>> {
        if self::versions(&header) != versions {
            return Ok(None);
        }
        let len = len(&header)?;
        Ok(Some(Self { header, body, len }))
    }
    pub fn hash(&self) -> [u8; 32] {
        hash(&self.header)
    }
    pub fn state(&self) -> Result<State> {
        let amounts = match &self.body {
            Body::Paged { dir, pages } => {
                let slots = read_slots(dir, self.len)?;
                let page_len = |page: usize| PAGE.min(self.len - page * PAGE);
                let mut amounts = vec![Amount::default(); self.len];
                // Scattered updates fragment the directory: read the pages in file order, in
                // bounded spans across small runs of garbage slots, then put each in its place.
                let mut order: Vec<usize> = (0..slots.len()).collect();
                order.sort_unstable_by_key(|&page| slots[page]);
                let mut span = Vec::new();
                let mut i = 0;
                while i < order.len() {
                    let first = slots[order[i]];
                    let mut j = i + 1;
                    while j < order.len()
                        && slots[order[j]] - slots[order[j - 1]] <= MAX_GAP
                        && slots[order[j]] - first < MAX_SPAN
                    {
                        j += 1;
                    }
                    let last = order[j - 1];
                    span.resize(
                        (slots[last] - first) as usize * PAGE + page_len(last),
                        Amount::default(),
                    );
                    read_amounts(pages, first as u64 * PAGE_BYTES, &mut span)?;
                    for &page in &order[i..j] {
                        let at = (slots[page] - first) as usize * PAGE;
                        let n = page_len(page);
                        amounts[page * PAGE..page * PAGE + n].copy_from_slice(&span[at..at + n]);
                    }
                    i = j;
                }
                amounts
            }
            Body::Packed {
                file,
                offset,
                bytes,
            } => {
                let mut bytes = vec![0; *bytes];
                file.read_exact_at(&mut bytes, *offset)?;
                let end = usize::try_from(word(&self.header, 64))
                    .ok()
                    .filter(|&n| n <= bytes.len())
                    .ok_or_else(|| invalid("invalid snapshot sections"))?;
                let mut sats = vec![0u64; self.len];
                let mut counts = vec![0u64; self.len];
                for (input, target) in [(&bytes[..end], &mut sats), (&bytes[end..], &mut counts)] {
                    let progress = simple_decompress_into(input, target)
                        .map_err(|_| invalid("invalid snapshot compression"))?;
                    if progress.n_processed != self.len || !progress.finished {
                        return Err(invalid("invalid snapshot length"));
                    }
                }
                sats.into_iter()
                    .zip(counts)
                    .map(|(sats, count)| Amount { sats, count })
                    .collect()
            }
        };
        State::new(amounts, self.hash())
    }
}
