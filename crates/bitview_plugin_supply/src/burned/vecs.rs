use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use bitview_vecs::ValuePerBlockCumulativeRolling;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub total: ValuePerBlockCumulativeRolling<M>,
}
