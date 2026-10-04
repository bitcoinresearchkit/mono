use bitview_primitives::{Bytes, Weight64};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockRolling, PerBlockRolling};
use brk_types::{Height, VSize, Weight};
use vecdb::{LazyVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Virtual size in vbytes: block weight in weight units divided by four
    /// and rounded up. Cumulative values divide the cumulative weight and
    /// rolling sums and averages are taken from them, so they can differ from
    /// summed per-block values by less than one vbyte per block.
    pub(super) vbytes: VBytes,
    /// Total serialized block size in bytes, including the header,
    /// transaction-count CompactSize, and witness data.
    pub size: PerBlockRolling<Bytes, M>,
}

#[derive(Clone, Traversable)]
pub(super) struct VBytes {
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub(super) block: LazyVec<Height, VSize, Height, Weight>,
    #[traversable(flatten)]
    pub(super) rolling: LazyPerBlockRolling<VSize, Weight64>,
}
