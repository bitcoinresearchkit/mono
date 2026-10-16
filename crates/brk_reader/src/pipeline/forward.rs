#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::os::fd::AsRawFd;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read},
    ops::ControlFlow,
    os::unix::fs::FileExt,
    path::Path,
    sync::OnceLock,
    thread,
    time::Duration,
};

use bitcoin::block::Header;
use brk_error::{Error, Result};
use brk_types::{BlkMetadata, Height, ReadBlock};
use crossbeam::channel::{Receiver, Sender, bounded};
use parking_lot::{Condvar, Mutex};
use rustc_hash::FxHashSet;
use tracing::error;

use crate::{
    BlkIndexToBlkPath, XORBytes, XORIndex,
    canonical::CanonicalRange,
    parse::{parse_canonical_body, peek_canonical},
    pipeline::{CHANNEL_CAPACITY, reorder::ReorderState},
    scan::scan_bytes,
};

/// Raw bytes of blocks dispatched to the parsers and not yet emitted in order. A
/// gap keeps the blocks past it decoded in the reorder buffer, so this bounds
/// that buffer; blocks found beyond the budget keep only their position and are
/// re-read once emissions free it. The block the reorder stage waits for always
/// goes through. Headers-first sync leaves up to ~1.2 GiB (raw) past a gap; at
/// this budget the re-reads leave read time unchanged.
pub(super) const MAX_OUTSTANDING_BYTES: usize = 256 * 1024 * 1024;

struct ScannedBlock {
    frame: Frame,
    bytes: Vec<u8>,
    canonical_offset: u32,
}

/// A canonical block's location and already-decoded header.
struct Frame {
    metadata: BlkMetadata,
    len: usize,
    header: Header,
}

/// Reader-side ordering: every canonical offset below `next` is dispatched.
struct Dispatch<'a> {
    paths: &'a BlkIndexToBlkPath,
    parser_send: &'a Sender<ScannedBlock>,
    reorder: &'a Mutex<ReorderState>,
    emitted: &'a Condvar,
    stop: &'a OnceLock<Stop>,
    next: u32,
    /// Offsets past `next` already dispatched.
    dispatched: FxHashSet<u32>,
    /// Offsets found while the budget was spent, lowest first.
    deferred: BTreeMap<u32, Frame>,
}

impl Dispatch<'_> {
    /// `Ok(false)` once the parsers hung up.
    fn offer(&mut self, canonical_offset: u32, frame: Frame, bytes: &[u8]) -> Result<bool> {
        if canonical_offset < self.next
            || self.dispatched.contains(&canonical_offset)
            || self.deferred.contains_key(&canonical_offset)
        {
            return Ok(true);
        }
        if self.deferred.is_empty() && self.admit(canonical_offset, frame.len, false) {
            return Ok(self.send(canonical_offset, frame, bytes.to_vec()));
        }
        self.deferred.insert(canonical_offset, frame);
        self.drain(false)
    }

    /// Dispatches the deferred blocks, lowest first, while the budget allows; `wait`s for
    /// emissions to free it (until stopped) instead of returning. `Ok(false)` once the parsers
    /// hung up.
    fn drain(&mut self, wait: bool) -> Result<bool> {
        while let Some((&offset, frame)) = self.deferred.first_key_value() {
            // An offset below never turned up: emission can't reach this one, and the
            // pipeline reports the missing block when it finishes.
            if wait && self.next < offset {
                return Ok(true);
            }
            if !self.admit(offset, frame.len, wait) {
                return Ok(true);
            }
            let frame = self
                .deferred
                .remove(&offset)
                .expect("the first deferred block");
            let bytes = self.reread(&frame)?;
            if !self.send(offset, frame, bytes) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn admit(&self, offset: u32, len: usize, wait: bool) -> bool {
        let mut reorder = self.reorder.lock();
        while !reorder.admit(offset, len) {
            if !wait || self.stop.get().is_some() {
                return false;
            }
            self.emitted
                .wait_for(&mut reorder, Duration::from_millis(10));
        }
        true
    }

    fn reread(&self, frame: &Frame) -> Result<Vec<u8>> {
        let position = frame.metadata.position();
        let path = self
            .paths
            .get(&position.blk_index())
            .ok_or(Error::Internal(
                "forward pipeline: deferred block's file vanished",
            ))?;
        let mut bytes = vec![0u8; frame.len];
        File::open(path)?.read_exact_at(&mut bytes, u64::from(position.offset()))?;
        Ok(bytes)
    }

    fn send(&mut self, canonical_offset: u32, frame: Frame, bytes: Vec<u8>) -> bool {
        self.dispatched.insert(canonical_offset);
        while self.dispatched.remove(&self.next) {
            self.next += 1;
        }
        self.parser_send
            .send(ScannedBlock {
                frame,
                bytes,
                canonical_offset,
            })
            .is_ok()
    }
}

enum Stop {
    Done,
    Failed(Error),
}

/// A third of the cores, at most four: on 12 cores, four parsers stream recent blocks to a light
/// consumer ~1.5x faster than one and cost a CPU-bound consumer (the indexer) nothing, while
/// eight cost it ~10%. Small machines keep one parser and leave their cores to the consumer.
fn parser_threads() -> usize {
    thread::available_parallelism().map_or(1, |cores| (cores.get() / 3).clamp(1, 4))
}

pub fn pipeline_forward(
    paths: &BlkIndexToBlkPath,
    first_blk_index: u16,
    xor_bytes: XORBytes,
    canonical: &CanonicalRange,
    send: &Sender<Result<ReadBlock>>,
) -> Result<()> {
    let (parser_send, parser_recv) = bounded::<ScannedBlock>(CHANNEL_CAPACITY);
    let reorder = Mutex::new(ReorderState::new(send.clone()));
    let emitted = Condvar::new();
    let stop: OnceLock<Stop> = OnceLock::new();

    thread::scope(|scope| {
        for _ in 0..parser_threads() {
            let parser_recv = parser_recv.clone();
            scope.spawn(|| {
                parser_loop(parser_recv, &reorder, &emitted, &stop, canonical, xor_bytes)
            });
        }
        drop(parser_recv);

        let read_result = read_and_dispatch(
            paths,
            first_blk_index,
            xor_bytes,
            canonical,
            Dispatch {
                paths,
                parser_send: &parser_send,
                reorder: &reorder,
                emitted: &emitted,
                stop: &stop,
                next: 0,
                dispatched: FxHashSet::default(),
                deferred: BTreeMap::new(),
            },
        );
        drop(parser_send);
        read_result
    })?;

    if let Some(Stop::Failed(e)) = stop.into_inner() {
        return Err(e);
    }
    reorder.into_inner().finalize(canonical.len())
}

fn parser_loop(
    parser_recv: Receiver<ScannedBlock>,
    reorder: &Mutex<ReorderState>,
    emitted: &Condvar,
    stop: &OnceLock<Stop>,
    canonical: &CanonicalRange,
    xor_bytes: XORBytes,
) {
    for ScannedBlock {
        frame: Frame {
            metadata, header, ..
        },
        bytes,
        canonical_offset,
    } in parser_recv
    {
        if stop.get().is_some() {
            continue;
        }
        let height = Height::from(*canonical.start + canonical_offset);
        let len = bytes.len();
        let xor_state = XORIndex::at_offset(metadata.position().offset() as usize);
        let block =
            match parse_canonical_body(bytes, metadata, xor_state, xor_bytes, height, header) {
                Ok(block) => block,
                Err(e) => {
                    error!("parse_canonical_body failed at height {height}: {e}");
                    let _ = stop.set(Stop::Failed(e));
                    continue;
                }
            };
        let pipeline_finished = {
            let mut state = reorder.lock();
            !state.try_emit(canonical_offset, block, len)
                || state.next_offset as usize >= canonical.len()
        };
        emitted.notify_one();
        if pipeline_finished {
            let _ = stop.set(Stop::Done);
        }
    }
}

fn read_and_dispatch(
    paths: &BlkIndexToBlkPath,
    first_blk_index: u16,
    xor_bytes: XORBytes,
    canonical: &CanonicalRange,
    mut dispatch: Dispatch<'_>,
) -> Result<()> {
    let stop = dispatch.stop;
    for (&blk_index, blk_path) in paths.range(first_blk_index..) {
        if stop.get().is_some() {
            return Ok(());
        }
        let mut bytes = read_uncached(blk_path)?;
        scan_bytes(
            &mut bytes,
            blk_index,
            xor_bytes,
            |metadata, block_bytes, xor_state| {
                if stop.get().is_some() {
                    return ControlFlow::Break(());
                }
                let (canonical_offset, header) =
                    match peek_canonical(block_bytes, xor_state, xor_bytes, canonical) {
                        Ok(Some(found)) => found,
                        Ok(None) => return ControlFlow::Continue(()),
                        Err(e) => {
                            let _ = stop.set(Stop::Failed(e));
                            return ControlFlow::Break(());
                        }
                    };
                debug_assert_eq!(
                    xor_state,
                    XORIndex::at_offset(metadata.position().offset() as usize)
                );
                let frame = Frame {
                    metadata,
                    len: block_bytes.len(),
                    header,
                };
                match dispatch.offer(canonical_offset, frame, block_bytes) {
                    Ok(true) => ControlFlow::Continue(()),
                    Ok(false) => ControlFlow::Break(()),
                    Err(e) => {
                        let _ = stop.set(Stop::Failed(e));
                        ControlFlow::Break(())
                    }
                }
            },
        );
    }
    if stop.get().is_none() {
        dispatch.drain(true)?;
    }
    Ok(())
}

/// Reads a blk file once and keeps it out of the OS page cache (macOS reads it uncached, Linux
/// drops it right after): caching 128 MiB per file would evict the data consumers look up while
/// indexing.
fn read_uncached(path: &Path) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    // SAFETY: an advisory flag on an open descriptor; a failure only leaves caching on.
    #[cfg(target_os = "macos")]
    unsafe {
        libc::fcntl(file.as_raw_fd(), libc::F_NOCACHE, 1);
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    // SAFETY: advice on an open descriptor; a failure only leaves the pages cached.
    #[cfg(target_os = "linux")]
    unsafe {
        libc::posix_fadvise(file.as_raw_fd(), 0, 0, libc::POSIX_FADV_DONTNEED);
    }
    Ok(bytes)
}
