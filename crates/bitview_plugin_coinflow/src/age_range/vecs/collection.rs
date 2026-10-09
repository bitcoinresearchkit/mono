use bitview_cohort::AgeRange;
use bitview_primitives::{BoundedRatio, Float64, PerDay, Ratio64};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlock, LazySpotValuePerBlock, PerBlock};
use brk_types::Height;
use vecdb::{ReadableVec, Rw, StorageMode};

use super::SupplyVecs;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub ranges: AgeRange<RangeVecs<M>>,
}

/// Coinflow of one UTXO age range.
#[derive(Traversable)]
pub struct RangeVecs<M: StorageMode = Rw> {
    /// Empirical daily spending hazard: cumulative transfer volume in BTC
    /// divided by cumulative coin days created in the range. It estimates the
    /// fraction of the range's supply spent per day; higher values indicate
    /// faster turnover. Returns zero when cumulative coin days created is zero.
    pub spending_rate: PerBlock<PerDay, M>,
    /// Estimated remaining-lifetime spending exposure. It integrates observed
    /// positive spending hazards from the range midpoint through subsequent
    /// complete ranges, then integrates an exponential tail fitted by
    /// duration-weighted regression of log hazard on age. Returns zero when a
    /// decreasing finite tail cannot be fitted. Larger exposure implies a
    /// greater eventual probability of spending.
    pub spending_exposure: PerBlock<Float64, M>,
    /// Mobility: estimated probability that supply in the range will ever be
    /// spent, one minus exp of negative spending exposure. Nonpositive or NaN
    /// exposure returns zero; positive results are capped just below one. A
    /// value near zero identifies supply unlikely to move, while a value near
    /// one identifies supply likely to move eventually. The source is floored
    /// at bounded scale 4,294,967,294.
    pub mobility: LazyPerBlock<Ratio64, BoundedRatio>,
    pub supply: SupplyVecs<LazySpotValuePerBlock>,
    #[traversable(hidden)]
    pub mobility_source: CachedSeries<Height, BoundedRatio, M>,
}

impl<M: StorageMode> Vecs<M> {
    /// Mobility by height for each age range: the coinflow URPD weight source.
    pub fn urpd_weight_sources(&self) -> AgeRange<&impl ReadableVec<Height, Ratio64>> {
        AgeRange::from_fn(|id| &id.select(&self.ranges).mobility.height)
    }
}
