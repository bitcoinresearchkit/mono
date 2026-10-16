pub mod block;
pub mod buffer;
pub mod transaction;
pub mod txin;
pub mod txout;

pub use buffer::BlockBuffers;

use brk_types::{Height, ReadBlock};

use crate::{Lengths, Readers, Stores, Vecs};

/// Processes a single block, extracting and storing all indexed data.
pub struct BlockProcessor<'a> {
    pub block: &'a ReadBlock,
    pub height: Height,
    pub lengths: &'a mut Lengths,
    pub vecs: &'a mut Vecs,
    pub stores: &'a mut Stores,
    pub readers: &'a Readers,
}
