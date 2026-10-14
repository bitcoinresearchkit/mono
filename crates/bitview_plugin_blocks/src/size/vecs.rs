use bitview_primitives::Bytes32;
use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use crate::block_rolling::BlockRolling;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Serialized block size in bytes, including witness data: its 80-byte header,
    /// transaction-count CompactSize, and every serialized transaction.
    pub size: BlockRolling<Bytes32, M>,
}
