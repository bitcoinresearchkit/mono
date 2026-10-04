use bitview_cohort::SpendableType;
use bitview_primitives::{Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlockCumulativeRolling, LazyPercentCumulativeRolling};
use brk_types::Height;
use vecdb::{Rw, StorageMode};

use super::WithInputTypes;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Counts of transaction inputs. The `all` aggregate includes one coinbase
    /// input per block. Per-type series exclude coinbase and classify inputs by
    /// the BRK output type of the previous output they spend; `OP_RETURN` is
    /// excluded because it is unspendable.
    pub input_count: WithInputTypes<LazyPerBlockCumulativeRolling<Count>>,
    /// Inputs spending a previous-output type divided by all inputs
    /// over the same cumulative or trailing window. The denominator includes
    /// coinbase inputs.
    pub(crate) input_share: SpendableType<LazyPercentCumulativeRolling<PartsPerMillion32>>,
    /// Number of non-coinbase transactions containing at least one input that
    /// spends a previous-output type. Each transaction is counted
    /// once per type; the `all` aggregate counts every non-coinbase transaction.
    pub(crate) tx_count: WithInputTypes<LazyPerBlockCumulativeRolling<Count>>,
    /// Non-coinbase transactions containing a previous-output type
    /// divided by all non-coinbase transactions over the same cumulative or
    /// trailing window.
    pub(crate) tx_share: SpendableType<LazyPercentCumulativeRolling<PartsPerMillion32>>,
    #[traversable(hidden)]
    pub(crate) input_count_stored: SpendableType<CachedSeries<Height, Count, M>>,
    #[traversable(hidden)]
    pub(crate) tx_count_stored: SpendableType<CachedSeries<Height, Count, M>>,
}
