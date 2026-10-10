use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use bitview_vecs::ValuePerBlockCumulativeRolling;

#[derive(Traversable)]
#[traversable(transparent)]
pub struct Vecs<M: StorageMode = Rw> {
    pub total: ValuePerBlockCumulativeRolling<M>,
}
