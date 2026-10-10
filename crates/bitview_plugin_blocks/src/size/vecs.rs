use bitview_primitives::Bytes32;
use bitview_traversable::Traversable;
use bitview_vecs::LazyPerBlockRolling;
use brk_types::{Height, VSize, Weight};
use vecdb::{LazyVec, Rw, StorageMode};

use crate::block_rolling::BlockRolling;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Virtual size in vbytes: block weight in weight units divided by four
    /// and rounded up. Cumulative values divide the cumulative weight and
    /// rolling sums and averages are taken from them, so they can differ from
    /// summed per-block values by less than one vbyte per block.
    pub(super) vsize: VirtualSize,
    /// Total serialized block size in bytes, including the header,
    /// transaction-count CompactSize, and witness data.
    pub size: BlockRolling<Bytes32, M>,
}

#[derive(Clone, Traversable)]
pub(super) struct VirtualSize {
    /// Value for the represented block.
    pub(super) block: LazyVec<Height, VSize, Height, Weight>,
    #[traversable(flatten)]
    pub(super) rolling: LazyPerBlockRolling<VSize, Weight>,
}
