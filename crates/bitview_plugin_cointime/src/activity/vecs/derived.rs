use bitview_primitives::{BoundedRatio, StoredF64};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, PerBlock};
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct DerivedVecs<M: StorageMode = Rw> {
    /// Liveliness: cumulative coinblocks destroyed divided by cumulative
    /// coinblocks created. A value nearer one means more of the holding time
    /// accumulated by the supply has been consumed by spending; a value nearer
    /// zero means more remains stored in unspent outputs. Stored at bounded
    /// scale 4,294,967,294 with downward quantization; exposed as a decimal.
    pub liveliness: LazyPerBlock<StoredF64, BoundedRatio>,
    /// One minus liveliness, where liveliness is cumulative coinblocks
    /// destroyed divided by cumulative coinblocks created. A value nearer one
    /// means more accumulated holding time remains stored in unspent outputs.
    /// Derived by exact encoded complement of the bounded liveliness source.
    pub vaultedness: LazyPerBlock<StoredF64, BoundedRatio>,
    /// Ratio of consumed to still-stored holding time: liveliness divided by
    /// vaultedness, or `liveliness / (1 - liveliness)`. Values above one mean
    /// consumed holding time exceeds stored holding time.
    pub ratio: LazyPerBlock<StoredF64, BoundedRatio>,
    /// Canonical bounded source; cumulative coinblock inputs remain unrounded.
    #[traversable(hidden)]
    pub liveliness_source: PerBlock<BoundedRatio, M>,
}
