use bitview_traversable::Traversable;
use brk_types::StoredU64;
use vecdb::{Rw, StorageMode};

use bitview_compute::PerBlockAggregated;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Number of transaction outputs, including coinbase outputs.
    pub total: PerBlockAggregated<StoredU64, M>,
}
