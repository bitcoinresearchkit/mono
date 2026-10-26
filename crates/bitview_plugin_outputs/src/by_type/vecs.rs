use bitview_cohort::ByType;
use bitview_primitives::{Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{CumulativeSource, LazyPerBlockCumulativeRolling, LazyPercentCumulativeRolling};
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub(crate) types: ByType<OutputTypeVecs>,
    #[traversable(hidden)]
    pub(crate) count_stored: ByType<CumulativeSource<Count, M>>,
    #[traversable(hidden)]
    pub(crate) tx_count_stored: ByType<CumulativeSource<Count, M>>,
}

#[derive(Clone, Traversable)]
pub struct OutputTypeVecs {
    /// Number of outputs of the type.
    #[traversable(lead = "Outputs of this type.")]
    pub(crate) count: LazyPerBlockCumulativeRolling<Count>,
    /// Outputs of the type divided by all outputs over the same cumulative or
    /// trailing window.
    #[traversable(lead = "This type's share of outputs.")]
    pub(crate) share: LazyPercentCumulativeRolling<PartsPerMillion32>,
    /// Number of transactions with at least one output of the type.
    #[traversable(lead = "Transactions creating this type.")]
    pub(crate) tx_count: LazyPerBlockCumulativeRolling<Count>,
    /// Transactions with at least one output of the type divided by all
    /// transactions over the same cumulative or trailing window.
    #[traversable(lead = "Transactions creating this type, as a share of all transactions.")]
    pub(crate) tx_share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}
