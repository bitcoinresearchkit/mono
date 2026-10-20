use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use crate::flagged::TxCount;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Transactions whose version is exactly 1.
    pub v1: TxCount<M>,
    /// Transactions whose version is exactly 2.
    pub v2: TxCount<M>,
    /// Transactions whose version is exactly 3.
    pub v3: TxCount<M>,
    /// Transactions whose version is not 1, 2, or 3.
    pub other: TxCount<M>,
}
