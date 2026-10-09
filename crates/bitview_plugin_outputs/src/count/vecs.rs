use bitview_primitives::Count32;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockFullFromCumulative;
use derive_more::{Deref, DerefMut};
use vecdb::{Rw, StorageMode};

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw>(
    /// Number of transaction outputs, including coinbase outputs.
    #[traversable(flatten)]
    pub PerBlockFullFromCumulative<Count32, M>,
);
