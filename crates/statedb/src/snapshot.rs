use crate::{Amount, State, util::invalid};
use pco::{
    ChunkConfig, DeltaSpec, ModeSpec,
    standalone::{simple_compress, simple_decompress_into},
};
use std::{fs::File, io::Result, os::unix::fs::FileExt};
use zerocopy::IntoBytes;
const RAW: &[u8; 8] = b"STATER04";
const PCO: &[u8; 8] = b"STATEP04";
const HEADER: usize = 72;
pub(crate) const MAX_RECORD: u64 = 2 * 1024 * 1024 * 1024;

pub(crate) fn write(
    file: &File,
    offset: u64,
    state: &State,
    versions: (u64, u64),
    compressed: bool,
) -> Result<u64> {
    let mut header = [0; HEADER];
    header[..8].copy_from_slice(if compressed { PCO } else { RAW });
    header[8..16].copy_from_slice(&(state.len() as u64).to_le_bytes());
    header[16..48].copy_from_slice(&state.hash());
    header[48..56].copy_from_slice(&versions.0.to_le_bytes());
    header[56..64].copy_from_slice(&versions.1.to_le_bytes());
    if compressed {
        let sats: Vec<_> = state.amounts().iter().map(|v| v.sats).collect();
        let counts: Vec<_> = state.amounts().iter().map(|v| v.count).collect();
        let config = ChunkConfig::default()
            .with_compression_level(3)
            .with_mode_spec(ModeSpec::Classic)
            .with_delta_spec(DeltaSpec::NoOp);
        let sats =
            simple_compress(&sats, &config).map_err(|_| invalid("sats compression failed"))?;
        let counts =
            simple_compress(&counts, &config).map_err(|_| invalid("counts compression failed"))?;
        let bytes = (HEADER + sats.len() + counts.len()) as u64;
        if bytes > MAX_RECORD {
            return Err(invalid("snapshot exceeds page limit"));
        }
        header[64..72].copy_from_slice(&(sats.len() as u64).to_le_bytes());
        file.write_all_at(&header, offset)?;
        file.write_all_at(&sats, offset + HEADER as u64)?;
        file.write_all_at(&counts, offset + HEADER as u64 + sats.len() as u64)?;
        Ok(bytes)
    } else {
        let bytes = (HEADER + state.len() * 16) as u64;
        if bytes > MAX_RECORD {
            return Err(invalid("snapshot exceeds page limit"));
        }
        file.write_all_at(&header, offset)?;
        #[cfg(target_endian = "little")]
        file.write_all_at(state.amounts().as_bytes(), offset + HEADER as u64)?;
        #[cfg(target_endian = "big")]
        {
            let mut body = Vec::with_capacity(state.len() * 16);
            for v in state.amounts() {
                body.extend_from_slice(&v.encode());
            }
            file.write_all_at(&body, offset + HEADER as u64)?;
        }
        Ok(bytes)
    }
}
/// Opening a snapshot reads its identity; reconstruction reads its payload once.
pub(crate) struct Snapshot {
    file: File,
    offset: u64,
    bytes: usize,
    header: [u8; HEADER],
    pub len: usize,
}
impl Snapshot {
    pub fn open(
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
        if &header[..8] != RAW && &header[..8] != PCO {
            return Err(invalid("unsupported snapshot format"));
        }
        let word = |i| u64::from_le_bytes(header[i..i + 8].try_into().unwrap());
        if (word(48), word(56)) != versions {
            return Ok(None);
        };
        let len = usize::try_from(word(8)).map_err(|_| invalid("snapshot length overflow"))?;
        if len > 100_000_000 {
            return Err(invalid("snapshot exceeds height limit"));
        }
        Ok(Some(Self {
            file: file.try_clone()?,
            offset: offset + HEADER as u64,
            bytes: bytes - HEADER,
            header,
            len,
        }))
    }
    pub fn hash(&self) -> [u8; 32] {
        self.header[16..48].try_into().unwrap()
    }
    pub fn state(&self) -> Result<State> {
        if &self.header[..8] == RAW {
            if self.bytes != self.len * 16 {
                return Err(invalid("invalid raw snapshot length"));
            }
            let mut amounts = vec![Amount::default(); self.len];
            self.file
                .read_exact_at(amounts.as_mut_bytes(), self.offset)?;
            #[cfg(target_endian = "big")]
            for v in &mut amounts {
                v.sats = u64::from_le(v.sats);
                v.count = u64::from_le(v.count);
            }
            return State::new(amounts, self.hash());
        }
        let mut bytes = vec![0; self.bytes];
        self.file.read_exact_at(&mut bytes, self.offset)?;
        let end = usize::try_from(u64::from_le_bytes(self.header[64..72].try_into().unwrap()))
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
        let amounts = sats
            .into_iter()
            .zip(counts)
            .map(|(sats, count)| Amount { sats, count })
            .collect();
        State::new(amounts, self.hash())
    }
}
