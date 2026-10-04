use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockCumulativeRolling;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Transactions whose version is exactly 1.
    pub v1: PerBlockCumulativeRolling<Count, M>,
    /// Transactions whose version is exactly 2.
    pub v2: PerBlockCumulativeRolling<Count, M>,
    /// Transactions whose version is exactly 3.
    pub v3: PerBlockCumulativeRolling<Count, M>,
    /// Transactions whose version is not 1, 2, or 3. This category combines
    /// every other value; use individual raw transaction data to inspect the
    /// original version.
    pub other: PerBlockCumulativeRolling<Count, M>,
}
