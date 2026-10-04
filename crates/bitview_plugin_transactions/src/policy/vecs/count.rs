use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockCumulativeRolling;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct CountVecs<M: StorageMode = Rw> {
    /// Number of transactions classified as nonstandard under this
    /// approximation.
    pub nonstandard: PerBlockCumulativeRolling<Count, M>,
}
