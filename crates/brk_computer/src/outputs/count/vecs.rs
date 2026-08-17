use brk_traversable::Traversable;
use brk_types::StoredU64;
use vecdb::{Rw, StorageMode};

use crate::internal::PerBlockAggregated;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Number of transaction outputs, including coinbase outputs.
    pub total: PerBlockAggregated<StoredU64, M>,
}
