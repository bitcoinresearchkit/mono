use std::vec;

use brk_error::Result;
use brk_reader::{BlockReceiver, Reader};
use brk_rpc::Client;
use brk_types::{BlockHash, Height};

pub enum State {
    Rpc {
        client: Client,
        heights: vec::IntoIter<Height>,
        prev_hash: Option<BlockHash>,
    },
    Reader {
        receiver: BlockReceiver,
    },
}

impl State {
    pub fn new_rpc(
        client: Client,
        start: Height,
        end: Height,
        prev_hash: Option<BlockHash>,
    ) -> Self {
        let heights = (*start..=*end)
            .map(Height::new)
            .collect::<Vec<_>>()
            .into_iter();

        Self::Rpc {
            client,
            heights,
            prev_hash,
        }
    }

    /// `after_hash` selects between the two Reader entry points:
    ///
    /// * `Some(anchor)` → [`Reader::after`], which seeds the pipeline's
    ///   continuity check with `anchor` so the very first emitted
    ///   block is verified against it. This is what stops a stale
    ///   anchor (tip of a reorged-out chain) from silently producing
    ///   a stitched stream.
    /// * `None` → [`Reader::range`], which has no anchor to verify
    ///   against and just streams the canonical blocks at the given
    ///   heights.
    pub fn new_reader(
        reader: Reader,
        start: Height,
        end: Height,
        after_hash: Option<BlockHash>,
    ) -> Result<Self> {
        let receiver = match after_hash {
            Some(hash) => reader.after(Some(hash))?,
            None => reader.range(start, end)?,
        };
        Ok(State::Reader { receiver })
    }
}
