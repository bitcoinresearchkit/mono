use bitview_cohort::SpendableType;
use bitview_primitives::{Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{CumulativeSource, LazyPerBlockCumulativeRolling, LazyPercentCumulativeRolling};
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub(crate) types: SpendableType<InputTypeVecs>,
    #[traversable(hidden)]
    pub(crate) count_stored: SpendableType<CumulativeSource<Count, M>>,
    #[traversable(hidden)]
    pub(crate) tx_count_stored: SpendableType<CumulativeSource<Count, M>>,
}

#[derive(Clone, Traversable)]
pub struct InputTypeVecs {
    /// Number of inputs spending outputs of the type.
    pub(crate) count: LazyPerBlockCumulativeRolling<Count>,
    /// Inputs spending outputs of the type divided by all non-coinbase inputs
    /// over the same cumulative or trailing window.
    pub(crate) share: LazyPercentCumulativeRolling<PartsPerMillion32>,
    /// Number of transactions with at least one input spending an output of
    /// the type.
    pub(crate) tx_count: LazyPerBlockCumulativeRolling<Count>,
    /// Transactions with at least one input spending an output of the type
    /// divided by all non-coinbase transactions over the same cumulative or
    /// trailing window.
    pub(crate) tx_share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}
