//! Derive, store and expose daily URPD metrics.
pub(super) mod bounds;
mod compute;
mod cost_basis;
mod density;
mod import;
mod snapshots;

use std::path::PathBuf;

use bitview_cohort::UTXOAggregate;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyDailyPriceWithRatio};
use brk_types::{Cents, Day1};
use vecdb::{AnyStoredVec, Rw, StorageMode};

use cost_basis::CostBasisMetrics;
use density::DensityMetrics;

const WRITE_INTERVAL_DAYS: usize = 100;

#[derive(Traversable)]
pub struct Metrics<M: StorageMode = Rw> {
    /// Daily cost-basis percentiles from weighted URPD buckets, weighted by coin value or capital.
    pub cost_basis: UTXOAggregate<CostBasisMetrics<M>>,
    /// Capital-weighted mean of rounded URPD prices; distinct from exact per-output moments.
    pub capitalized_price: UTXOAggregate<LazyDailyPriceWithRatio>,
    /// Share of each cohort's weighted supply within 5% of daily closing spot.
    pub supply_density: DensityMetrics<M>,
    #[traversable(hidden)]
    capitalized_price_stored: UTXOAggregate<CachedSeries<Day1, Cents, M>>,
    #[traversable(skip)]
    states_path: PathBuf,
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
