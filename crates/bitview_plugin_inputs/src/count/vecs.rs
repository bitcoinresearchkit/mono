use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockAggregated;
use derive_more::{Deref, DerefMut};
use vecdb::{Rw, StorageMode};

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw>(
    /// Number of transaction inputs, including one coinbase input per block.
    #[traversable(flatten)]
    pub PerBlockAggregated<Count, M>,
);
