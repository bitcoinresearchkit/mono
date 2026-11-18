//! One pipeline for every range: locate each block by its header, hopping the blk files frame to
//! frame, and read the bodies in chain order. Core writes blocks out of height order, so a block
//! found before its turn waits as a position (its file, offset and header), never as bytes.

#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::os::fd::AsRawFd;
use std::{fs::File, io, mem, os::unix::fs::FileExt, path::Path, sync::Arc, thread};

use bitcoin::block::Header;
use brk_error::{Error, Result};
use brk_types::{BlkPosition, Height, ReadBlock};
use crossbeam_channel::{Receiver, Sender, bounded};
use parking_lot::{Condvar, Mutex};
use rustc_hash::FxHashMap;
use tracing::error;

use crate::{
    BlockReceiver, ReaderInner, XORBytes, XORIndex,
    canonical::CanonicalRange,
    parse::{parse_canonical_body, read_header},
    reader_inner::AppendPosition,
    scan::frames,
};

const CHANNEL_CAPACITY: usize = 50;

/// How many files before the anchor's the hop starts: during initial sync and reindex Core
/// writes blocks out of height order, a later block up to 16 files before an earlier one's on
/// the nodes measured (2026-11), where this margin left 11 files to spare at worst.
const OUT_OF_ORDER_FILES: usize = 21;

/// A block of the range, located by its header.
struct Found {
    file: Arc<BlkFile>,
    position: BlkPosition,
    len: usize,
    header: Header,
}

/// A stream that reports `error` as its only item.
pub fn failed(error: Error) -> BlockReceiver {
    let (send, recv) = bounded(1);
    let _ = send.send(Err(error));
    recv.into_iter()
}

/// Streams `canonical`'s blocks in chain order. A failure (an unreadable file, a block no file
/// holds, a parse error) is the stream's last item.
pub fn spawn(reader: Arc<ReaderInner>, canonical: CanonicalRange) -> BlockReceiver {
    let (send, recv) = bounded(CHANNEL_CAPACITY);
    if !canonical.is_empty() {
        thread::spawn(move || {
            if let Err(e) = stream(&reader, &canonical, &send) {
                let _ = send.send(Err(e));
            }
        });
    }
    recv.into_iter()
}

fn stream(
    reader: &ReaderInner,
    canonical: &CanonicalRange,
    send: &Sender<Result<ReadBlock>>,
) -> Result<()> {
    let (located_send, located_recv) = bounded(CHANNEL_CAPACITY);
    let turn = &Turn::default();
    thread::scope(|scope| {
        let parsers: Vec<_> = (0..parser_threads())
            .map(|_| {
                let located = located_recv.clone();
                scope.spawn(move || parse_in_turn(reader, canonical, located, turn, send))
            })
            .collect();
        drop(located_recv);
        let located = locate(reader, canonical, located_send);
        parsers.into_iter().fold(located, |result, parser| {
            let parsed = parser.join();
            result.and(parsed.unwrap_or(Err(Error::Internal("a block parser panicked"))))
        })
    })
}

/// A third of the cores, at most four: on 12 cores, four parsers stream recent blocks to a light
/// consumer ~1.5x faster than one and cost a CPU-bound consumer (the indexer) nothing, while
/// eight cost it ~10%. Small machines keep one parser and leave their cores to the consumer.
fn parser_threads() -> usize {
    thread::available_parallelism().map_or(1, |cores| (cores.get() / 3).clamp(1, 4))
}

/// Hands the range's blocks to the parsers in chain order, each as soon as it and every block
/// before it are located.
fn locate(
    reader: &ReaderInner,
    canonical: &CanonicalRange,
    send: Sender<(u32, Found)>,
) -> Result<()> {
    let mut located = Located {
        canonical,
        xor_bytes: reader.xor_bytes,
        next: 0,
        early: FxHashMap::default(),
        send,
    };
    // An update right after the previous one: its blocks are appended after the last one
    // streamed, in the same file or, once Core rolls over, the next.
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
    // Any other start, and what the append leaves missing (a rollover, a block written before
    // its parent, files rewritten by a reindex).
    let paths = reader.refresh_paths()?;
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
    Err(Error::Internal("blk files missing canonical blocks"))
}

/// Where in `files` to hop from for the blocks after one mined at `anchor_time`: the last file
/// whose first block is no newer, then [`OUT_OF_ORDER_FILES`] back. First blocks' times only
/// roughly ascend, which the margin covers too.
fn first_file(files: &[(u16, &Path)], xor_bytes: XORBytes, anchor_time: u32) -> Result<usize> {
    let mut failed = None;
    let no_newer = files.partition_point(|(_, path)| {
        first_block_time(path, xor_bytes)
            .map(|time| time.is_some_and(|time| time <= anchor_time))
            .unwrap_or_else(|e| {
                failed.get_or_insert(e);
                false
            })
    });
    match failed {
        Some(e) => Err(e.into()),
        None => Ok(no_newer.saturating_sub(1 + OUT_OF_ORDER_FILES)),
    }
}

/// The header time of a blk file's first block, `None` while it holds none.
fn first_block_time(path: &Path, xor_bytes: XORBytes) -> io::Result<Option<u32>> {
    let file = File::open(path)?;
    let Some(frame) = frames(&file, xor_bytes, 0)?.next() else {
        return Ok(None);
    };
    Ok(read_header(&file, frame?, xor_bytes)?.map(|header| header.time))
}

/// The range's blocks as their frames are found, in any order.
struct Located<'a> {
    canonical: &'a CanonicalRange,
    xor_bytes: XORBytes,
    /// Every canonical offset below it is sent.
    next: u32,
    /// Blocks found before their turn.
    early: FxHashMap<u32, Found>,
    send: Sender<(u32, Found)>,
}

impl Located<'_> {
    /// Hops the frames of `files`, the first one from `offset` (a frame boundary), and sends the
    /// range's blocks as their turns come. True once the last is sent or the parsers hung up;
    /// false when the files end with blocks still missing.
    fn hop<'p>(
        &mut self,
        files: impl IntoIterator<Item = (u16, &'p Path)>,
        mut offset: usize,
    ) -> Result<bool> {
        for (blk_index, path) in files {
            let start = mem::take(&mut offset);
            let file = match BlkFile::open(path) {
                Ok(file) => Arc::new(file),
                // Pruned or renumbered since it was listed: its blocks, if any were needed,
                // are reported missing.
                Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            };
            for frame in frames(&file.cached, self.xor_bytes, start)? {
                if self.offer(&file, blk_index, frame?)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Takes the frame if it holds a block of the range not found yet, reading its header only.
    /// True once the last block is sent or the parsers hung up.
    fn offer(
        &mut self,
        file: &Arc<BlkFile>,
        blk_index: u16,
        frame: (usize, usize),
    ) -> Result<bool> {
        let Some(header) = read_header(&file.cached, frame, self.xor_bytes)? else {
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
                file: file.clone(),
                position: BlkPosition::new(blk_index, offset as u32),
                len,
                header,
            },
        );
        while let Some(found) = self.early.remove(&self.next) {
            if self.send.send((self.next, found)).is_err() {
                return Ok(true);
            }
            self.next += 1;
        }
        Ok(self.next as usize == self.canonical.len())
    }
}

/// The canonical offset whose block is sent next. Parsers finish in any order and each waits for
/// its block's turn, so they hold one decoded block each at most. `None` once the stream
/// stopped: the consumer left, or a parser failed.
struct Turn {
    next: Mutex<Option<u32>>,
    passed: Condvar,
}

impl Default for Turn {
    fn default() -> Self {
        Self {
            next: Mutex::new(Some(0)),
            passed: Condvar::new(),
        }
    }
}

/// Stops the stream when a parser leaves before the located blocks end (an error, a panic, the
/// consumer gone): the others would wait for its block's turn for good.
struct StopOnDrop<'a>(&'a Turn);

impl Drop for StopOnDrop<'_> {
    fn drop(&mut self) {
        *self.0.next.lock() = None;
        self.0.passed.notify_all();
    }
}

/// Reads and parses located blocks, sends each at its turn, and retains where the range's last
/// block ends for the next call's append hop.
fn parse_in_turn(
    reader: &ReaderInner,
    canonical: &CanonicalRange,
    located: Receiver<(u32, Found)>,
    turn: &Turn,
    send: &Sender<Result<ReadBlock>>,
) -> Result<()> {
    let stop = StopOnDrop(turn);
    for (slot, found) in located {
        let height = Height::from(*canonical.start + slot);
        let block = read_block(&found, reader.xor_bytes, height)
            .inspect_err(|e| error!("Block {height} at {} failed to read: {e}", found.position))?;
        let hash = *block.hash();
        let mut next = turn.next.lock();
        while next.is_some_and(|next| next != slot) {
            turn.passed.wait(&mut next);
        }
        if next.is_none() {
            return Ok(());
        }
        if send.send(Ok(block)).is_err() {
            return Ok(());
        }
        *next = Some(slot + 1);
        drop(next);
        turn.passed.notify_all();
        if slot as usize + 1 == canonical.len() {
            reader.retain_append_position(AppendPosition {
                hash,
                blk_index: found.position.blk_index(),
                end: found.position.offset() as usize + found.len,
            });
        }
    }
    // The located blocks ended: the other parsers still send the ones they hold.
    mem::forget(stop);
    Ok(())
}

fn read_block(found: &Found, xor_bytes: XORBytes, height: Height) -> Result<ReadBlock> {
    let offset = found.position.offset();
    parse_canonical_body(
        found.file.read_uncached(offset, found.len)?,
        found.position,
        XORIndex::at_offset(offset as usize),
        xor_bytes,
        height,
        found.header,
    )
}

/// A blk file open twice. The frame hop reads through the OS page cache: its read-ahead makes a
/// run of small blocks one read (uncached, each 88-byte read is a device round trip, and the
/// chain's first 200,000 blocks streamed 5x slower on an M6). Blocks are read past the cache
/// (macOS reads that descriptor uncached, Linux drops each block right after its read): each is
/// read once, and caching them all would evict the data consumers look up while indexing.
struct BlkFile {
    cached: File,
    uncached: File,
}

impl BlkFile {
    fn open(path: &Path) -> io::Result<Self> {
        let uncached = File::open(path)?;
        // SAFETY: an advisory flag on an open descriptor; a failure only leaves caching on.
        #[cfg(target_os = "macos")]
        unsafe {
            libc::fcntl(uncached.as_raw_fd(), libc::F_NOCACHE, 1);
        }
        Ok(Self {
            cached: File::open(path)?,
            uncached,
        })
    }

    fn read_uncached(&self, offset: u32, len: usize) -> io::Result<Vec<u8>> {
        let mut bytes = vec![0u8; len];
        self.uncached.read_exact_at(&mut bytes, u64::from(offset))?;
        // SAFETY: advice on an open descriptor; a failure only leaves the pages cached.
        #[cfg(target_os = "linux")]
        unsafe {
            libc::posix_fadvise(
                self.uncached.as_raw_fd(),
                libc::off_t::from(offset),
                len as libc::off_t,
                libc::POSIX_FADV_DONTNEED,
            );
        }
        Ok(bytes)
    }
}
