use bitview_cohort::ByType;
use bitview_primitives::{Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyFixedRatioCumulativeRolling, LazyPerBlockCumulativeRolling};
use brk_types::Height;
use vecdb::{Rw, StorageMode};

use super::{SpendableOutputCount, WithOutputTypes};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Counts of transaction outputs, including coinbase. Per-type series
    /// classify outputs by BRK locking-script type.
    pub output_count: WithOutputTypes<LazyPerBlockCumulativeRolling<Count>>,
    /// Number of transaction outputs excluding `OP_RETURN` outputs, which are
    /// provably unspendable.
    pub spendable_output_count: SpendableOutputCount,
    /// Outputs of a BRK locking-script type divided by all outputs
    /// over the same cumulative or trailing window, including coinbase outputs.
    pub(crate) output_share: ByType<LazyFixedRatioCumulativeRolling<PartsPerMillion32>>,
    /// Number of transactions containing at least one output of a
    /// BRK locking-script type. Each transaction is counted once per type; the
    /// `all` aggregate counts every transaction, including coinbase.
    pub(crate) tx_count: WithOutputTypes<LazyPerBlockCumulativeRolling<Count>>,
    /// Transactions containing an output type divided by all
    /// transactions over the same cumulative or trailing window, including
    /// coinbase transactions.
    pub(crate) tx_share: ByType<LazyFixedRatioCumulativeRolling<PartsPerMillion32>>,
    #[traversable(hidden)]
    pub(crate) output_count_stored: ByType<CachedSeries<Height, Count, M>>,
    #[traversable(hidden)]
    pub(crate) tx_count_stored: ByType<CachedSeries<Height, Count, M>>,
}
