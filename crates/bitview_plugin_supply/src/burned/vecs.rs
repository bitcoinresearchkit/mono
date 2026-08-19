use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use bitview_compute::ValuePerBlockCumulative;

#[derive(Traversable)]
#[traversable(transparent)]
pub struct Vecs<M: StorageMode = Rw> {
    pub total: ValuePerBlockCumulative<M>,
}
