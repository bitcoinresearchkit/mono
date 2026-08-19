use bitview_traversable::Traversable;
use brk_types::StoredU64;
use vecdb::{Rw, StorageMode};

use bitview_compute::PerBlock;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Number of transaction outputs tracked as unspent after each block.
    pub count: PerBlock<StoredU64, M>,
}
