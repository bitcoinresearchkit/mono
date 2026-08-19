use bitview_traversable::Traversable;
use brk_cohort::{AgeRange, AgeRangeId};
use brk_types::{Sats, StoredF64};
use vecdb::{Rw, StorageMode};

use bitview_compute::{
    ColumnarPerBlock, ColumnarPerBlockCumulativeRolling, LazyColumnPerBlockCumulativeRolling,
    LazyColumnSpotValuePerBlock,
};

use super::{ActivitySeries, SupplyVecs};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Coin days destroyed by spent outputs, allocated across every age range
    /// the outputs traversed. The portion above a spent output's age-range
    /// lower bound remains in that range; each fully traversed younger range
    /// receives spent BTC multiplied by that range's duration. The allocation
    /// preserves total coin days destroyed.
    pub coindays_consumed: ColumnarPerBlockCumulativeRolling<
        StoredF64,
        AgeRangeId,
        AgeRange<LazyColumnPerBlockCumulativeRolling<StoredF64, AgeRangeId>>,
        M,
    >,
    /// Cumulative coin days created in each age range minus cumulative coin
    /// days consumed from that range.
    pub coindays_stored: ColumnarPerBlockCumulativeRolling<
        StoredF64,
        AgeRangeId,
        AgeRange<LazyColumnPerBlockCumulativeRolling<StoredF64, AgeRangeId>>,
        M,
    >,
    /// Wakefulness for each UTXO age range: cumulative coin days consumed from
    /// the range divided by cumulative coin days created in the range.
    pub activity: ColumnarPerBlock<StoredF64, AgeRangeId, ActivitySeries, M>,
    pub supply: SupplyVecs<
        ColumnarPerBlock<Sats, AgeRangeId, AgeRange<LazyColumnSpotValuePerBlock<AgeRangeId>>, M>,
    >,
}
