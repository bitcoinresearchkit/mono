use std::sync::Arc;

use brk_error::Result;
use brk_reader::Reader;
use brk_rpc::Client;
use brk_types::Height;

mod iterator;
mod source;
mod state;

use iterator::BlockIterator;
use source::Source;
use state::State;

/// Selects RPC for small ranges and the block reader for larger ones.
/// Iteration can fail when a chain reorganization breaks continuity.
#[derive(Clone)]
pub struct Blocks(Arc<Source>);

impl Blocks {
    pub fn new(client: &Client, reader: &Reader) -> Self {
        Self(Arc::new(Source {
            client: client.clone(),
            reader: reader.clone(),
        }))
    }

    /// Iterate over an inclusive range. Reversed ranges are empty without I/O.
    pub fn range(&self, start: Height, end: Height) -> Result<BlockIterator> {
        if start > end {
            return Ok(BlockIterator::new(State::Empty));
        }

        // An inclusive range can contain 2^32 heights.
        let count = u64::from(*end) - u64::from(*start) + 1;
        let state = if count <= 10 {
            State::new_rpc(self.0.client.clone(), start, end)
        } else {
            State::new_reader(self.0.reader.clone(), start, end)?
        };
        Ok(BlockIterator::new(state))
    }
}
