//! Derive, store and expose per-block URPD metrics.
pub(super) mod bounds;
mod compute;
mod cost_basis;
pub(crate) mod density;
mod import;
mod metric_buckets;
mod price_distribution;
mod price_stats;

use bitview_cohort::UTXOAggregate;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPriceWithRatioPerBlock};
use brk_types::{Cents, Height};
use vecdb::{AnyStoredVec, Rw, StorageMode};

use crate::Replay;
use cost_basis::CostBasisMetrics;
use density::DensityMetrics;
use metric_buckets::MetricBuckets;

const WRITE_INTERVAL_BLOCKS: usize = 10_000;

#[derive(Traversable)]
pub struct Metrics<M: StorageMode = Rw> {
    #[traversable(skip)]
    replay: M::WriteOnly<Replay>,
    #[traversable(skip)]
    buffer: M::WriteOnly<MetricBuckets>,
    /// Per-block cost-basis percentiles from weighted URPD buckets, weighted by coin value or capital.
    pub cost_basis: UTXOAggregate<CostBasisMetrics<M>>,
    /// Capital-weighted mean of rounded URPD prices; distinct from exact per-output moments.
    pub capitalized_price: UTXOAggregate<LazyPriceWithRatioPerBlock>,
    /// Share of each cohort's weighted supply within 5% of per-block closing spot.
    pub supply_density: DensityMetrics<M>,
    #[traversable(hidden)]
    capitalized_price_stored: UTXOAggregate<CachedSeries<Height, Cents, M>>,
}

impl Metrics {
    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.cost_basis
            .iter_mut()
            .flat_map(CostBasisMetrics::stored_vecs_mut)
            .chain(
                self.capitalized_price_stored
                    .iter_mut()
                    .map(|v| v as &mut dyn AnyStoredVec),
            )
            .chain(self.supply_density.stored_vecs_mut())
    }
}
