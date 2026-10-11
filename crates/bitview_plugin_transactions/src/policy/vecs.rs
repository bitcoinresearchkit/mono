use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use crate::flagged::Classified;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Transactions classified as nonstandard under this approximation.
    pub nonstandard: Classified<M>,
}
