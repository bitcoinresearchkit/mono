use bitview_primitives::StoredU64;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockCumulativeRolling;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Transactions whose version is exactly 1.
    pub v1: PerBlockCumulativeRolling<StoredU64, M>,
    /// Transactions whose version is exactly 2.
    pub v2: PerBlockCumulativeRolling<StoredU64, M>,
    /// Transactions whose version is exactly 3.
    pub v3: PerBlockCumulativeRolling<StoredU64, M>,
    /// Transactions whose version is not 1, 2, or 3. This category combines
    /// every other value; use individual raw transaction data to inspect the
    /// original version.
    pub other: PerBlockCumulativeRolling<StoredU64, M>,
}
