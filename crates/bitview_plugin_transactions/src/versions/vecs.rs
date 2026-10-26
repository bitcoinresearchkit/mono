use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use crate::flagged::TxCount;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Transactions whose version is exactly 1.
    #[traversable(lead = "Version 1 transactions.")]
    pub v1: TxCount<M>,
    /// Transactions whose version is exactly 2.
    #[traversable(lead = "Version 2 transactions.")]
    pub v2: TxCount<M>,
    /// Transactions whose version is exactly 3.
    #[traversable(lead = "Version 3 transactions.")]
    pub v3: TxCount<M>,
    /// Transactions whose version is not 1, 2, or 3.
    #[traversable(lead = "Transactions of any other version.")]
    pub other: TxCount<M>,
}
