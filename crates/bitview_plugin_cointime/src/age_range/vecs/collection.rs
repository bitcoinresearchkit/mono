use bitview_cohort::AgeRange;
use bitview_primitives::{BoundedRatio, CoinDays, Ratio64};
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

/// Cointime of one UTXO age range.
#[derive(Traversable)]
pub struct RangeVecs<M: StorageMode = Rw> {
    /// Coin days created in the range: its supply held for the block's
    /// duration (the age plugin's series).
    pub coindays_created: LazyPerBlockCumulativeRolling<CoinDays>,
    /// Coin days destroyed by spent outputs, allocated across every age range
    /// the outputs traversed. The portion above a spent output's age-range
    /// lower bound remains in that range; each fully traversed younger range
    /// receives spent BTC multiplied by that range's duration. The allocation
    /// preserves total coin days destroyed. Unlike the age range's coin days
    /// destroyed, which credit a spent output's whole age to the range it was
    /// spent from, this takes coin days from the ranges where they were
    /// created, so consumption never exceeds creation in any range.
    pub coindays_consumed: PerBlockCumulativeRolling<CoinDays, M>,
    /// Cumulative coin days created in the range minus cumulative coin days
    /// consumed from it.
    pub coindays_stored: PerBlockCumulativeRolling<CoinDays, M>,
    /// Wakefulness: cumulative coin days consumed from the range divided by
    /// cumulative coin days created in it. Higher values mean more of the
    /// holding time accumulated in the range has been consumed by spending.
    /// The source is floored at bounded scale 4,294,967,294; cumulative coin-day
    /// inputs remain full precision.
    pub wakefulness: LazyPerBlock<Ratio64, BoundedRatio>,
    /// Awake supply divided by dormant supply, `wakefulness / (1 - wakefulness)`:
    /// above one, more of the range's holding time was consumed than stored.
    pub awake_to_dormant: LazyPerBlock<Ratio64, BoundedRatio>,
    /// The range weighted by its wakefulness.
    pub awake: SideVecs,
    /// The range weighted by one minus its wakefulness.
    pub dormant: SideVecs,
    #[traversable(hidden)]
    pub wakefulness_source: CachedSeries<Height, BoundedRatio, M>,
}

impl<M: StorageMode> Vecs<M> {
    /// Wakefulness by height for each age range: the cointime URPD weight source.
    pub fn urpd_weight_sources(&self) -> AgeRange<&impl ReadableVec<Height, Ratio64>> {
        AgeRange::from_fn(|id| &id.select(&self.ranges).wakefulness.height)
    }

    pub(crate) fn wakefulness_sources(&self) -> AgeRange<&CachedSeries<Height, BoundedRatio, M>> {
        AgeRange::from_fn(|id| &id.select(&self.ranges).wakefulness_source)
    }
}
