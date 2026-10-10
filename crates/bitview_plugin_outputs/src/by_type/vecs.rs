use bitview_cohort::ByType;
use bitview_primitives::{Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlockCumulativeRolling, LazyPercentCumulativeRolling};
use brk_types::Height;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub(crate) types: ByType<OutputTypeVecs>,
    #[traversable(hidden)]
    pub(crate) count_stored: ByType<CachedSeries<Height, Count, M>>,
    #[traversable(hidden)]
    pub(crate) tx_count_stored: ByType<CachedSeries<Height, Count, M>>,
}

/// Counts and shares of one output type.
#[derive(Clone, Traversable)]
pub struct OutputTypeVecs {
    /// Number of outputs of the type.
    pub(crate) count: LazyPerBlockCumulativeRolling<Count>,
    /// Outputs of the type divided by all outputs over the same cumulative or
    /// trailing window.
    pub(crate) share: LazyPercentCumulativeRolling<PartsPerMillion32>,
    /// Number of transactions with at least one output of the type; each
    /// transaction counts once.
    pub(crate) tx_count: LazyPerBlockCumulativeRolling<Count>,
    /// Transactions with at least one output of the type divided by all
    /// transactions over the same cumulative or trailing window.
    pub(crate) tx_share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}
