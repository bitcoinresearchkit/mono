//! One pipeline for every range: locate each block by its header, hopping the blk files frame to
//! frame, and read the bodies in chain order. Core writes blocks out of height order, so a block
//! found before its turn waits as a position (its file, offset and header), never as bytes.

#[cfg(target_os = "macos")]
use std::os::fd::AsRawFd;
use std::{fs::File, io, mem, os::unix::fs::FileExt, path::Path, sync::Arc, thread};

use bitcoin::block::Header;
use brk_error::{Error, Result};
use brk_types::{BlkPosition, BlockHash, Height, ReadBlock};
use crossbeam_channel::{Receiver, Sender, bounded};
use rustc_hash::FxHashMap;

use crate::{
    BlkIndexToBlkPath, ReaderInner, XORBytes, XORIndex,
    canonical::CanonicalRange,
    parse::{parse_canonical_body, read_header},
    reader_inner::AppendPosition,
    scan::frames,
};

/// How many blocks are read and parsed ahead of the consumer.
const BLOCKS_AHEAD: usize = 50;

/// How many files before the anchor's the hop starts. During initial sync Core writes blocks out
/// of height order: on the two nodes measured (2026-11) a later block sits up to 16 files before
/// an earlier one's.
const OUT_OF_ORDER_FILES: usize = 21;

/// A block of the range, located by its header.
struct Found {
    height: Height,
    position: BlkPosition,
    len: usize,
    header: Header,
}

/// One block's answer, from the parser that read it.
type Parsed = Receiver<Result<ReadBlock>>;

/// A range's blocks in chain order, read and parsed off the consumer's thread. The first error
/// (an unreadable file, a block no file holds, a parse error) is the last item.
pub struct BlockReceiver {
    /// The blocks' answers in chain order; `None` once an error ended the stream.
    in_order: Option<Receiver<Result<Parsed>>>,
}

impl Iterator for BlockReceiver {
    type Item = Result<ReadBlock>;

    fn next(&mut self) -> Option<Self::Item> {
        let block = self.in_order.as_ref()?.recv().ok()?.and_then(|parsed| {
            parsed
                .recv()
                .unwrap_or(Err(Error::Internal("a block parser panicked")))
        });
        if block.is_err() {
            // Hangs up on the pipeline, which stops.
            self.in_order = None;
        }
        Some(block)
    }
}

/// A stream that reports `error` as its only item.
pub fn failed(error: Error) -> BlockReceiver {
    let (send, in_order) = bounded(1);
    let _ = send.send(Err(error));
    BlockReceiver {
        in_order: Some(in_order),
    }
}

/// Streams `canonical`'s blocks in chain order.
pub fn spawn(reader: Arc<ReaderInner>, canonical: CanonicalRange) -> BlockReceiver {
    let (send, in_order) = bounded(BLOCKS_AHEAD);
    if !canonical.is_empty() {
        thread::spawn(move || {
            let reader = &*reader;
            let (to_parsers, located) = bounded(BLOCKS_AHEAD);
            thread::scope(|scope| {
                for _ in 0..parser_threads() {
                    let located = located.clone();
                    scope.spawn(move || parse(reader, located));
                }
                drop(located);
                if let Err(e) = locate(reader, &canonical, to_parsers, &send) {
                    let _ = send.send(Err(e));
                }
            });
        });
    }
    BlockReceiver {
        in_order: Some(in_order),
    }
}

/// A third of the cores, at most four: on 12 cores, four parsers stream recent blocks to a light
/// consumer ~1.5x faster than one and cost a CPU-bound consumer (the indexer) nothing, while
/// eight cost it ~10%. Small machines keep one parser and leave their cores to the consumer.
fn parser_threads() -> usize {
    thread::available_parallelism().map_or(1, |cores| (cores.get() / 3).clamp(1, 4))
}

/// Hands the range's blocks to the parsers in chain order, each as soon as it and every block
/// before it are located, and queues their answers for the consumer in the same order.
fn locate(
    reader: &ReaderInner,
    canonical: &CanonicalRange,
    to_parsers: Sender<(Found, Sender<Result<ReadBlock>>)>,
    in_order: &Sender<Result<Parsed>>,
) -> Result<()> {
    let mut located = Located {
        reader,
        canonical,
        next: 0,
        early: FxHashMap::default(),
        to_parsers,
        in_order,
    };
    // An update right after the previous one: its blocks are appended after the last one
    // located, in the same file or, once Core rolls over, the next.
    if let Some(position) = reader.append_position()
        && canonical.anchor() == Some(&position.hash)
    {
        let files = [position.blk_index, position.blk_index + 1]
            .map(|blk_index| (blk_index, reader.blk_path(blk_index)));
        let files = files.iter().map(|(blk_index, path)| (*blk_index, path.as_path()));
        if located.hop(files, position.end)? {
            return Ok(());
        }
    }
    // Any other start, and what the append leaves missing: a block written before its parent,
    // or files replaced by a resync, which raw reads then open again too.
    reader.close_blk_files();
    let paths = BlkIndexToBlkPath::scan(&reader.blocks_dir)?;
    let files: Vec<_> = paths
        .iter()
        .map(|(&blk_index, path)| (blk_index, path.as_path()))
        .collect();
    let first = match canonical.anchor_time() {
        Some(time) => first_file(&files, reader.xor_bytes, time)?,
        None => 0,
    };
    if located.hop(files[first..].iter().copied(), 0)? {
        return Ok(());
    }
    Err(Error::NotFound(format!(
        "Block {} is in none of the blk files",
        *canonical.start + located.next
    )))
}

/// Where in `files` to hop from for the blocks after one mined at `anchor_time`: a file whose
/// first block is no newer, right before one whose first block is newer, then
/// [`OUT_OF_ORDER_FILES`] back. First blocks' times only roughly ascend, so several files can fit:
/// the search never settles after the last of them, and an earlier one only hops more files (13
/// more at most on the two nodes measured, 2026-11).
fn first_file(files: &[(u16, &Path)], xor_bytes: XORBytes, anchor_time: u32) -> io::Result<usize> {
    let mut low = 0;
    let mut high = files.len();
    while low < high {
        let middle = low + (high - low) / 2;
        if first_block_time(files[middle].1, xor_bytes)?.is_some_and(|time| time <= anchor_time) {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    Ok(low.saturating_sub(1 + OUT_OF_ORDER_FILES))
}

/// The header time of a blk file's first block, `None` while it holds none.
fn first_block_time(path: &Path, xor_bytes: XORBytes) -> io::Result<Option<u32>> {
    let Some(file) = open(path)? else {
        return Ok(None);
    };
    let Some(frame) = frames(&file, xor_bytes, 0)?.next() else {
        return Ok(None);
    };
    Ok(read_header(&file, frame?, xor_bytes)?.map(|header| header.time))
}

/// `None` for a file that is gone, pruned since it was listed: its blocks, if any were needed,
/// are reported missing.
fn open(path: &Path) -> io::Result<Option<File>> {
    match File::open(path) {
        Ok(file) => Ok(Some(file)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// The range's blocks as their frames are found, in any order.
struct Located<'a> {
    reader: &'a ReaderInner,
    canonical: &'a CanonicalRange,
    /// Every canonical offset below it is handed over.
    next: u32,
    /// Blocks found before their turn.
    early: FxHashMap<u32, Found>,
    to_parsers: Sender<(Found, Sender<Result<ReadBlock>>)>,
    in_order: &'a Sender<Result<Parsed>>,
}

impl Located<'_> {
    /// Hops the frames of `files`, the first one from `offset` (a frame boundary), and hands
    /// over the range's blocks as their turns come. True once the last is handed over or the
    /// consumer hung up; false when the files end with blocks still missing. The frames are read
    /// through the OS page cache: its read-ahead makes a run of small blocks one read.
    fn hop<'p>(
        &mut self,
        files: impl IntoIterator<Item = (u16, &'p Path)>,
        mut offset: usize,
    ) -> Result<bool> {
        for (blk_index, path) in files {
            let start = mem::take(&mut offset);
            let Some(file) = open(path)? else {
                continue;
            };
            for frame in frames(&file, self.reader.xor_bytes, start)? {
                if self.offer(&file, blk_index, frame?)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Takes the frame if it holds a block of the range not found yet, reading its header only.
    /// True once the last block is handed over or the consumer hung up.
    fn offer(&mut self, file: &File, blk_index: u16, frame: (usize, usize)) -> Result<bool> {
        let Some(header) = read_header(file, frame, self.reader.xor_bytes)? else {
            return Ok(false);
        };
        let Some(slot) = self.canonical.offset_of(&header)? else {
            return Ok(false);
        };
        // Core may hold a block twice: the first copy found is the one read.
        if slot < self.next || self.early.contains_key(&slot) {
            return Ok(false);
        }
        let (offset, len) = frame;
        self.early.insert(
            slot,
            Found {
                height: Height::from(*self.canonical.start + slot),
                position: BlkPosition::new(blk_index, offset as u32),
                len,
                header,
            },
        );
        while let Some(found) = self.early.remove(&self.next) {
            self.next += 1;
            let last = self.next as usize == self.canonical.len();
            if last {
                // Where the next update's blocks are appended.
                self.reader.retain_append_position(AppendPosition {
                    hash: BlockHash::from(found.header.block_hash()),
                    blk_index: found.position.blk_index(),
                    end: found.position.offset() as usize + found.len,
                });
            }
            let (answer, parsed) = bounded(1);
            if self.in_order.send(Ok(parsed)).is_err()
                || self.to_parsers.send((found, answer)).is_err()
                || last
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

/// Reads and parses the blocks handed over, each answered on its own channel: the parsers
/// finish in any order, and the consumer takes the answers in chain order.
fn parse(reader: &ReaderInner, located: Receiver<(Found, Sender<Result<ReadBlock>>)>) {
    // The last file read: a block mostly sits in the same one as the block before it.
    let mut last = None;
    for (found, answer) in located {
        let block = read_block(reader, &found, &mut last).map_err(|e| {
            Error::Parse(format!(
                "Block {} in blk{:05}.dat at {} failed to read: {e}",
                found.height,
                found.position.blk_index(),
                found.position.offset()
            ))
        });
        // The consumer may be gone.
        let _ = answer.send(block);
    }
}

fn read_block(
    reader: &ReaderInner,
    found: &Found,
    last: &mut Option<(u16, File)>,
) -> Result<ReadBlock> {
    let blk_index = found.position.blk_index();
    let file = match last.take() {
        Some((index, file)) if index == blk_index => file,
        _ => open_past_cache(&reader.blk_path(blk_index))?,
    };
    let (_, file) = last.insert((blk_index, file));
    let offset = found.position.offset();
    let mut bytes = vec![0u8; found.len];
    file.read_exact_at(&mut bytes, u64::from(offset))?;
    parse_canonical_body(
        bytes,
        found.position,
        XORIndex::at_offset(offset as usize),
        reader.xor_bytes,
        found.height,
        found.header,
    )
}

/// A blk file for reading blocks' bodies. On macOS its reads leave the OS page cache alone: a
/// sync reads each block once, and caching them all would evict the data its consumer looks up
/// while indexing. A block already cached (the tip's, just written by Core) is still served from
/// there. Elsewhere the cache keeps what it reads: dropping each block after its read would
/// also drop the tip's block, which its other readers want next.
fn open_past_cache(path: &Path) -> io::Result<File> {
    let file = File::open(path)?;
    // SAFETY: an advisory flag on an open descriptor; a failure only leaves caching on.
    #[cfg(target_os = "macos")]
    unsafe {
        libc::fcntl(file.as_raw_fd(), libc::F_NOCACHE, 1);
    }
    Ok(file)
}
