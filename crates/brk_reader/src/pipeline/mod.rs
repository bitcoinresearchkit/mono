use std::{sync::Arc, thread};

use brk_error::{Error, Result};
use brk_rpc::Client;
use brk_types::Height;
use crossbeam::channel::bounded;

use crate::{
    BlkIndexToBlkPath, BlockReceiver, ReaderInner, XORBytes, bisect, canonical::CanonicalRange,
};

mod forward;
mod reorder;
mod tail;

pub const CHANNEL_CAPACITY: usize = 50;

/// Forward pays the bisection + 21-file backoff (~2.7 GB of reads)
/// regardless of how few canonical blocks live in the window, so
/// tail wins for any catchup within this many files of the tip.
const TAIL_DISTANCE_FILES: usize = 8;

enum Strategy {
    /// Every block found appended after the last one streamed.
    Appended(Vec<tail::Found>),
    /// First-block heights of the newest files, newest first, as probed
    /// while picking (`None`: the probe failed); the tail walk reuses them.
    Tail {
        paths: BlkIndexToBlkPath,
        probed: Vec<Option<Height>>,
    },
    Forward {
        paths: BlkIndexToBlkPath,
        first_blk_index: u16,
    },
}

/// A stream that reports `error` as its only item.
pub fn failed(error: Error) -> BlockReceiver {
    let (send, recv) = bounded(1);
    let _ = send.send(Err(error));
    recv.into_iter()
}

pub fn spawn(reader: Arc<ReaderInner>, canonical: CanonicalRange) -> Result<BlockReceiver> {
    if canonical.is_empty() {
        return Ok(bounded(0).1.into_iter());
    }

    let strategy = match tail::find_appended(&reader, &canonical)? {
        Some(found) => Strategy::Appended(found),
        None => {
            let paths = reader.refresh_paths()?;
            pick_strategy(&reader.client, paths, reader.xor_bytes, canonical.start)
        }
    };

    let (send, recv) = bounded(CHANNEL_CAPACITY);

    thread::spawn(move || {
        let result = match strategy {
            Strategy::Appended(found) => tail::emit(&reader, found, &canonical, &send),
            Strategy::Tail { paths, probed } => {
                tail::pipeline_tail(&reader, &paths, &probed, &canonical, &send)
            }
            Strategy::Forward {
                paths,
                first_blk_index,
            } => forward::pipeline_forward(
                &paths,
                first_blk_index,
                reader.xor_bytes,
                &canonical,
                &send,
            ),
        };
        if let Err(e) = result {
            let _ = send.send(Err(e));
        }
    });

    Ok(recv.into_iter())
}

fn pick_strategy(
    client: &Client,
    paths: BlkIndexToBlkPath,
    xor_bytes: XORBytes,
    canonical_start: Height,
) -> Strategy {
    if canonical_start != Height::ZERO {
        let mut probed = Vec::with_capacity(TAIL_DISTANCE_FILES);
        for (_, path) in paths.iter().rev().take(TAIL_DISTANCE_FILES) {
            let height = bisect::first_block_height(client, path, xor_bytes).ok();
            probed.push(height);
            if height.is_some_and(|h| h <= canonical_start) {
                return Strategy::Tail { paths, probed };
            }
        }
    }
    Strategy::Forward {
        first_blk_index: bisect::find_start_blk_index(client, canonical_start, &paths, xor_bytes),
        paths,
    }
}
