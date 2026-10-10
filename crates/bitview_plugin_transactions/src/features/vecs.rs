use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use super::CountVecs;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub count: CountVecs<M>,
}
