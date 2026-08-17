use derive_more::{Deref, DerefMut};

use brk_traversable::Traversable;
use brk_types::StoredU64;
use vecdb::{Rw, StorageMode};

use crate::internal::PerBlockAggregated;

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw>(
    /// Number of transaction inputs, including one coinbase input per block.
    #[traversable(flatten)]
    pub PerBlockAggregated<StoredU64, M>,
);
