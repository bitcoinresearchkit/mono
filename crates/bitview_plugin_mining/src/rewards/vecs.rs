use bitview_traversable::Traversable;
use brk_types::{Height, PartsPerMillion32, PartsPerMillion64, Sats};
use vecdb::{EagerVec, PcoVec, Rw, StorageMode};

use bitview_compute::{
    CachedValuePerBlockFull, LazyPercentCumulativeRolling, LazyPercentRollingWindows,
    ValuePerBlockCumulative, ValuePerBlockCumulativeRolling,
};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Sum of the output values of the block's coinbase transaction. This is
    /// the miner reward actually assigned to coinbase outputs, not the total
    /// reward available.
    pub coinbase: ValuePerBlockCumulativeRolling<M>,
    /// Coinbase output value minus the block's total transaction fees. This is
    /// a derived subsidy component, not the scheduled consensus subsidy, and
    /// can be lower when the miner leaves some available reward unclaimed.
    pub subsidy: ValuePerBlockCumulativeRolling<M>,
    /// Sum of input value minus output value across the block's non-coinbase
    /// transactions.
    pub fees: CachedValuePerBlockFull<M>,
    /// Sum of the output values of the block's non-coinbase transactions,
    /// equivalently their total input value minus transaction fees. Reported
    /// in satoshis.
    pub output_volume: M::Stored<EagerVec<PcoVec<Height, Sats>>>,
    /// Portion of the available block reward not assigned to coinbase outputs:
    /// scheduled subsidy plus transaction fees minus coinbase output value.
    pub unclaimed: ValuePerBlockCumulative<M>,
    /// Transaction fees divided by coinbase output value. Cumulative variants
    /// use cumulative totals; rolling variants use totals within the trailing
    /// window.
    #[traversable(wrap = "fees", rename = "dominance")]
    pub fee_dominance: LazyPercentCumulativeRolling<PartsPerMillion32>,
    /// One minus fee dominance, equivalently the derived subsidy component
    /// divided by coinbase output value. Cumulative variants use cumulative
    /// totals; rolling variants use totals within the trailing window.
    #[traversable(wrap = "subsidy", rename = "dominance")]
    pub subsidy_dominance: LazyPercentCumulativeRolling<PartsPerMillion32>,
    /// Total transaction fees in the trailing window divided by the total
    /// derived subsidy component in the same window.
    #[traversable(wrap = "fees", rename = "to_subsidy")]
    pub fee_to_subsidy: LazyPercentRollingWindows<PartsPerMillion64>,
}
