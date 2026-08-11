use brk_traversable::Traversable;
use brk_types::StoredF64;
use vecdb::{Rw, StorageMode};

use crate::internal::{LazyPerBlock, PerBlock};

#[derive(Traversable)]
pub struct DerivedVecs<M: StorageMode = Rw> {
    pub liveliness: PerBlock<StoredF64, M>,
    pub vaultedness: LazyPerBlock<StoredF64>,
    pub ratio: LazyPerBlock<StoredF64>,
}
