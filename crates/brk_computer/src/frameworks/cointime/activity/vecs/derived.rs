use brk_traversable::Traversable;
use brk_types::StoredF64;
use vecdb::{Rw, StorageMode};

use crate::internal::{LazyPerBlock, PerBlock};

#[derive(Traversable)]
pub struct DerivedVecs<M: StorageMode = Rw> {
    /// Cumulative coinblocks destroyed divided by cumulative coinblocks
    /// created.
    pub liveliness: PerBlock<StoredF64, M>,
    /// One minus liveliness.
    pub vaultedness: LazyPerBlock<StoredF64>,
    /// Liveliness divided by vaultedness.
    pub ratio: LazyPerBlock<StoredF64>,
}
