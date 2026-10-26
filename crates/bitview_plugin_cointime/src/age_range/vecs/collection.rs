use bitview_cohort::AgeRange;
use bitview_primitives::{BoundedRatio, CoinDays, Multiple, Ratio};
use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, LazyPerBlock, LazyPerBlockCumulativeRolling, PerBlockCumulativeRolling,
};
use brk_types::Height;
use vecdb::{ReadableVec, Rw, StorageMode};

use super::SideVecs;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub ranges: AgeRange<RangeVecs<M>>,
}

#[derive(Traversable)]
pub struct RangeVecs<M: StorageMode = Rw> {
    /// Coin days created in the range: its supply held for the block's
    /// duration, supply crossing into an older range counted in each range for its time there.
    #[traversable(lead = "Coin days created: holding time added, one per bitcoin per day.")]
    pub coindays_created: LazyPerBlockCumulativeRolling<CoinDays>,
    /// Coin days destroyed by spent outputs, allocated across every age range
    /// the outputs traversed. The portion above a spent output's age-range
    /// lower bound remains in that range; each fully traversed younger range
    /// receives spent BTC multiplied by that range's duration. The allocation
    /// preserves total coin days destroyed. Unlike the age range's coin days
    /// destroyed, which credit a spent output's whole age to the range it was
    /// spent from, this takes coin days from the ranges where they were
    /// created, so consumption never exceeds creation in any range.
    #[traversable(lead = "Coin days consumed: holding time spent as these coins move.")]
    pub coindays_consumed: PerBlockCumulativeRolling<CoinDays, M>,
    /// Wakefulness: cumulative coin days consumed from the range divided by
    /// cumulative coin days created in it. Higher values mean more of the
    /// holding time accumulated in the range has been consumed by spending.
    /// Undefined (null) while the range has no coin days created, its coins
    /// then counting as dormant.
    #[traversable(lead = "Wakefulness: how much of the range's holding time has been spent.")]
    pub wakefulness: LazyPerBlock<Ratio, BoundedRatio>,
    /// Awake supply divided by dormant supply, `wakefulness / (1 - wakefulness)`:
    /// above one, more of the range's holding time was consumed than stored.
    #[traversable(lead = "Awake against dormant: holding time spent, for each unit still stored.")]
    pub awake_to_dormant: LazyPerBlock<Multiple, BoundedRatio>,
    /// The range weighted by its wakefulness.
    #[traversable(lead = "Awake part: these coins, by how much of their holding time has been spent.")]
    pub awake: SideVecs,
    /// The range weighted by one minus its wakefulness.
    #[traversable(lead = "Dormant part: these coins, by how much of their holding time is still stored.")]
    pub dormant: SideVecs,
    #[traversable(hidden)]
    pub(crate) wakefulness_source: CachedSeries<Height, BoundedRatio, M>,
}

impl<M: StorageMode> Vecs<M> {
    /// Wakefulness by height for each age range: the cointime URPD weight source.
    pub fn urpd_weight_sources(&self) -> AgeRange<&impl ReadableVec<Height, BoundedRatio>> {
        AgeRange::from_fn(|id| &id.select(&self.ranges).wakefulness_source)
    }

    pub(crate) fn wakefulness_sources(&self) -> AgeRange<&CachedSeries<Height, BoundedRatio, M>> {
        AgeRange::from_fn(|id| &id.select(&self.ranges).wakefulness_source)
    }
}
