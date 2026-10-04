use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockAggregated;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Number of transaction outputs, including coinbase outputs.
    pub total: PerBlockAggregated<Count, M>,
}
