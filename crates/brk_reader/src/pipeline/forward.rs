#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::os::fd::AsRawFd;
use std::{
    fs::File,
    io::{self, Read},
    ops::ControlFlow,
    path::Path,
    sync::OnceLock,
    thread,
};

use bitcoin::block::Header;
use brk_error::{Error, Result};
use brk_types::{BlkMetadata, Height, ReadBlock};
use crossbeam::channel::{Receiver, Sender, bounded};
use parking_lot::Mutex;
use tracing::error;

use crate::{
    BlkIndexToBlkPath, BlockHash, XORBytes, XORIndex,
    canonical::CanonicalRange,
    parse::{parse_canonical_body, peek_canonical},
    pipeline::{CHANNEL_CAPACITY, reorder::ReorderState},
    scan::scan_bytes,
};

struct ScannedBlock {
    metadata: BlkMetadata,
    bytes: Vec<u8>,
    xor_state: XORIndex,
    canonical_offset: u32,
    header: Header,
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
    let stop: OnceLock<Stop> = OnceLock::new();

    thread::scope(|scope| {
        for _ in 0..parser_threads() {
            let parser_recv = parser_recv.clone();
            scope.spawn(|| parser_loop(parser_recv, &reorder, &stop, canonical, xor_bytes));
        }
        drop(parser_recv);

        let read_result = read_and_dispatch(
            paths,
            first_blk_index,
            xor_bytes,
            canonical,
            &parser_send,
            &stop,
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
    stop: &OnceLock<Stop>,
    canonical: &CanonicalRange,
    xor_bytes: XORBytes,
) {
    for ScannedBlock {
        metadata,
        bytes,
        xor_state,
        canonical_offset,
        header,
    } in parser_recv
    {
        if stop.get().is_some() {
            continue;
        }
        let height = Height::from(*canonical.start + canonical_offset);
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
            !state.try_emit(canonical_offset, block)
                || state.next_offset as usize >= canonical.len()
        };
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
    parser_send: &Sender<ScannedBlock>,
    stop: &OnceLock<Stop>,
) -> Result<()> {
    for (&blk_index, blk_path) in paths.range(first_blk_index..) {
        if stop.get().is_some() {
            return Ok(());
        }
        let mut bytes = read_uncached(blk_path)?;
        scan_bytes(
            &mut bytes,
            blk_index,
            0,
            xor_bytes,
            |metadata, block_bytes, xor_state| {
                if stop.get().is_some() {
                    return ControlFlow::Break(());
                }
                let Some((canonical_offset, header)) =
                    peek_canonical(block_bytes, xor_state, xor_bytes, canonical)
                else {
                    return ControlFlow::Continue(());
                };
                if !canonical.verify_prev(canonical_offset, &BlockHash::from(header.prev_blockhash))
                {
                    let _ = stop.set(Stop::Failed(Error::Internal(
                        "forward pipeline: canonical batch stitched across a reorg",
                    )));
                    return ControlFlow::Break(());
                }
                let scanned = ScannedBlock {
                    metadata,
                    bytes: block_bytes.to_vec(),
                    xor_state,
                    canonical_offset,
                    header,
                };
                if parser_send.send(scanned).is_err() {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        );
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
