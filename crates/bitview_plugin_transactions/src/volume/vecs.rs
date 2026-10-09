use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerSecondWindows, ValuePerBlockCumulativeRolling};
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Sum of the input values of non-coinbase transactions. This equals their
    /// total output value plus transaction fees and is not adjusted to estimate
    /// economic payment volume.
    pub value: ValuePerBlockCumulativeRolling<M>,
    /// Transaction rate, including coinbase transactions.
    pub tx_per_second: LazyPerSecondWindows,
}
