use std::{sync::Arc, thread};

use brk_error::Result;
use brk_rpc::Client;
use brk_types::Height;
use crossbeam::channel::bounded;

use crate::{
    BlkIndexToBlkPath, BlockReceiver, ReaderInner, XORBytes, bisect, canonical::CanonicalRange,
};

mod forward;
mod reorder;
mod tail;

pub(crate) const CHANNEL_CAPACITY: usize = 50;

/// Forward pays the bisection + 21-file backoff (~2.7 GB of reads)
/// regardless of how few canonical blocks live in the window, so
/// tail wins for any catchup within this many files of the tip.
const TAIL_DISTANCE_FILES: usize = 8;

/// The indexer is CPU-bound on the consumer side, so 1 reader + 1
/// parser leaves the rest of the cores for it.
pub(crate) const DEFAULT_PARSER_THREADS: usize = 1;

enum Strategy {
    Tail,
    Forward { first_blk_index: u16 },
}

pub(crate) fn spawn(
    reader: Arc<ReaderInner>,
    canonical: CanonicalRange,
    parser_threads: usize,
) -> Result<BlockReceiver> {
    let parser_threads = parser_threads.clamp(1, CHANNEL_CAPACITY);

    if canonical.is_empty() {
        return Ok(crate::block_receiver::new(bounded(0).1));
    }

    let paths = reader.refresh_paths()?;
    let xor_bytes = reader.xor_bytes;
    let strategy = pick_strategy(&reader.client, &paths, xor_bytes, canonical.start);

    let (send, recv) = bounded(CHANNEL_CAPACITY);

    thread::spawn(move || {
        let result = match strategy {
            Strategy::Tail => {
                tail::pipeline_tail(&reader.client, &paths, xor_bytes, &canonical, &send)
            }
            Strategy::Forward { first_blk_index } => forward::pipeline_forward(
                &paths,
                first_blk_index,
                xor_bytes,
                &canonical,
                &send,
                parser_threads,
            ),
        };
        if let Err(e) = result {
            let _ = send.send(Err(e));
        }
    });

    Ok(crate::block_receiver::new(recv))
}

fn pick_strategy(
    client: &Client,
    paths: &BlkIndexToBlkPath,
    xor_bytes: XORBytes,
    canonical_start: Height,
) -> Strategy {
    if canonical_start != Height::ZERO
        && paths
            .iter()
            .rev()
            .take(TAIL_DISTANCE_FILES)
            .any(|(_, path)| {
                bisect::first_block_height(client, path, xor_bytes)
                    .is_ok_and(|h| h <= canonical_start)
            })
    {
        return Strategy::Tail;
    }
    Strategy::Forward {
        first_blk_index: bisect::find_start_blk_index(client, canonical_start, paths, xor_bytes),
    }
}
