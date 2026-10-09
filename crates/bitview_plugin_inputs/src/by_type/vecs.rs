use bitview_cohort::{ByAddrType, SpendableType};
use bitview_primitives::{Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlockCumulativeRolling, LazyPercentCumulativeRolling};
use brk_types::Height;
use vecdb::{LazyVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub(crate) types: SpendableType<InputTypeVecs>,
    #[traversable(hidden)]
    pub(crate) count_stored: SpendableType<CachedSeries<Height, Count, M>>,
    #[traversable(hidden)]
    pub(crate) tx_count_stored: SpendableType<CachedSeries<Height, Count, M>>,
}

/// Counts and shares of inputs spending one output type.
#[derive(Clone, Traversable)]
pub struct InputTypeVecs {
    /// Number of inputs spending outputs of the type.
    pub(crate) count: LazyPerBlockCumulativeRolling<Count>,
    /// Inputs spending outputs of the type divided by all non-coinbase inputs
    /// over the same cumulative or trailing window.
    pub(crate) share: LazyPercentCumulativeRolling<PartsPerMillion32>,
    /// Number of transactions with at least one input spending an output of
    /// the type; each transaction counts once.
    pub(crate) tx_count: LazyPerBlockCumulativeRolling<Count>,
    /// Transactions with at least one input spending an output of the type
    /// divided by all non-coinbase transactions over the same cumulative or
    /// trailing window.
    pub(crate) tx_share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}

impl Vecs {
    /// Cumulative input counts of the address types.
    pub fn addr_type_counts(&self) -> ByAddrType<LazyVec<Height, Count, Height, Count>> {
        ByAddrType::from_fn(|id| {
            self.types
                .get(id.output_type())
                .count
                .cumulative
                .height
                .clone()
        })
    }
}
