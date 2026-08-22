use bitview_traversable::Traversable;
use brk_types::StoredF64;
use vecdb::{Rw, StorageMode};

use bitview_compute::{LazyPerBlock, PerBlock};

#[derive(Traversable)]
pub struct DerivedVecs<M: StorageMode = Rw> {
    /// Liveliness: cumulative coinblocks destroyed divided by cumulative
    /// coinblocks created. A value nearer one means more of the holding time
    /// accumulated by the supply has been consumed by spending; a value nearer
    /// zero means more remains stored in unspent outputs.
    pub liveliness: PerBlock<StoredF64, M>,
    /// One minus liveliness, where liveliness is cumulative coinblocks
    /// destroyed divided by cumulative coinblocks created. A value nearer one
    /// means more accumulated holding time remains stored in unspent outputs.
    pub vaultedness: LazyPerBlock<StoredF64>,
    /// Ratio of consumed to still-stored holding time: liveliness divided by
    /// vaultedness, or `liveliness / (1 - liveliness)`. Values above one mean
    /// consumed holding time exceeds stored holding time.
    pub ratio: LazyPerBlock<StoredF64>,
}
