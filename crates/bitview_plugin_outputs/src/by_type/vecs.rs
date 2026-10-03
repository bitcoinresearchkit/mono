use bitview_cohort::ByType;
use bitview_primitives::{PartsPerMillion32, StoredU64};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlockCumulativeRolling, LazyPercentCumulativeRolling};
use brk_types::Height;
use vecdb::{Rw, StorageMode};

use super::{SpendableOutputCount, WithOutputTypes};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Counts of transaction outputs, including coinbase. Per-type series
    /// classify outputs by BRK locking-script type.
    pub output_count: WithOutputTypes<LazyPerBlockCumulativeRolling<StoredU64>>,
    /// Number of transaction outputs excluding `OP_RETURN` outputs, which are
    /// provably unspendable.
    pub spendable_output_count: SpendableOutputCount,
    /// Outputs of a BRK locking-script type divided by all outputs
    /// over the same cumulative or trailing window, including coinbase outputs.
    pub(crate) output_share: ByType<LazyPercentCumulativeRolling<PartsPerMillion32>>,
    /// Number of transactions containing at least one output of a
    /// BRK locking-script type. Each transaction is counted once per type; the
    /// `all` aggregate counts every transaction, including coinbase.
    pub(crate) tx_count: WithOutputTypes<LazyPerBlockCumulativeRolling<StoredU64>>,
    /// Transactions containing an output type divided by all
    /// transactions over the same cumulative or trailing window, including
    /// coinbase transactions.
    pub(crate) tx_share: ByType<LazyPercentCumulativeRolling<PartsPerMillion32>>,
    #[traversable(hidden)]
    pub(crate) output_count_stored: ByType<CachedSeries<Height, StoredU64, M>>,
    #[traversable(hidden)]
    pub(crate) tx_count_stored: ByType<CachedSeries<Height, StoredU64, M>>,
}
