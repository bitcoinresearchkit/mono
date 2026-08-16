use brk_traversable::Traversable;
use brk_types::StoredU64;
use vecdb::{Rw, StorageMode};

use crate::internal::PerBlockCumulativeRolling;

#[derive(Traversable)]
pub struct CountVecs<M: StorageMode = Rw> {
    /// Number of transactions classified as nonstandard under this
    /// approximation.
    pub nonstandard: PerBlockCumulativeRolling<StoredU64, M>,
}
