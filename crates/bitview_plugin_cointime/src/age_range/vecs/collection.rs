use bitview_cohort::AgeRange;
use bitview_primitives::{BoundedRatio, StoredF64};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazySpotValuePerBlock, PerBlockCumulativeRolling};
use brk_types::Height;
use vecdb::{ReadableVec, Rw, StorageMode};

use super::{ActivitySeries, SupplyVecs};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Coin days destroyed by spent outputs, allocated across every age range
    /// the outputs traversed. The portion above a spent output's age-range
    /// lower bound remains in that range; each fully traversed younger range
    /// receives spent BTC multiplied by that range's duration. The allocation
    /// preserves total coin days destroyed.
    pub coindays_consumed: AgeRange<PerBlockCumulativeRolling<StoredF64, M>>,
    /// Cumulative coin days created in each age range minus cumulative coin
    /// days consumed from that range.
    pub coindays_stored: AgeRange<PerBlockCumulativeRolling<StoredF64, M>>,
    /// Wakefulness for each UTXO age range: cumulative coin days consumed from
    /// the range divided by cumulative coin days created in the range. Higher
    /// values mean more of the holding time accumulated in that range has been
    /// consumed by spending. The source is floored at bounded scale
    /// 4,294,967,294; cumulative coin-day inputs remain full precision.
    pub activity: ActivitySeries,
    #[traversable(hidden)]
    pub activity_sources: AgeRange<CachedSeries<Height, BoundedRatio, M>>,
    pub supply: SupplyVecs<AgeRange<LazySpotValuePerBlock>>,
}

impl<M: StorageMode> Vecs<M> {
    /// Wakefulness by height for each age range: the cointime URPD weight source.
    pub fn urpd_weight_sources(&self) -> AgeRange<&impl ReadableVec<Height, StoredF64>> {
        AgeRange::from_fn(|id| &id.select(&self.activity.wakefulness).height)
    }
}
