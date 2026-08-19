mod block;
mod buffer;
mod transaction;
mod txin;
mod txout;

pub use buffer::BlockBuffers;

use brk_types::{Block, Height};

use crate::{Lengths, Readers, Stores, Vecs};

/// Processes a single block, extracting and storing all indexed data.
pub struct BlockProcessor<'a> {
    pub block: &'a Block,
    pub height: Height,
    pub check_collisions: bool,
    pub lengths: &'a mut Lengths,
    pub vecs: &'a mut Vecs,
    pub stores: &'a mut Stores,
    pub readers: &'a Readers,
}
