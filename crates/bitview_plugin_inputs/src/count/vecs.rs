use bitview_primitives::Count16;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockFullFromCumulative;
use derive_more::{Deref, DerefMut};
use vecdb::{Rw, StorageMode};

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw>(
    /// Number of transaction inputs, excluding coinbase inputs.
    #[traversable(flatten)]
    pub PerBlockFullFromCumulative<Count16, M>,
);
