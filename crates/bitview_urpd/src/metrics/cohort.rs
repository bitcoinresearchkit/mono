use bitview_cohort::AgeAggregateId;
use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, IndexSources, LazyPriceWithRatioPerBlock, import_cached};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode, WritableVec};

use super::{
    cost_basis::CostBasisMetrics,
    density::{DensitySeries, SupplyDensity},
    price_stats::PriceStats,
};

/// The same stored and derived metrics for every URPD age filter.
#[derive(Traversable)]
pub struct CohortMetrics<M: StorageMode = Rw> {
    /// Coin- and capital-weighted percentiles of rounded creation prices.
    pub cost_basis: CostBasisMetrics<M>,
    /// Capital-weighted mean of rounded creation prices.
    pub capitalized_price: LazyPriceWithRatioPerBlock,
    /// Weighted supply within 5% of closing spot, divided by this cohort's supply.
    pub supply_density: DensitySeries<M>,
    #[traversable(hidden)]
    capitalized_price_stored: CachedSeries<Height, Cents, M>,
}

impl CohortMetrics {
    pub fn import(
        db: &Database,
        owner: &str,
        cohort: AgeAggregateId,
        version: Version,
        indexes: &IndexSources,
        spot: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let name = format!("{owner}_urpd_{}", cohort.name());
        let capitalized_price_stored =
            import_cached(db, &format!("{name}_capitalized_price_cents"), version)?;
        Ok(Self {
            cost_basis: CostBasisMetrics::import(
                db,
                &cohort.metric_name(&format!("{owner}_cost_basis")),
                version,
                indexes,
            )?,
            capitalized_price: LazyPriceWithRatioPerBlock::from_height_source(
                &format!("{name}_capitalized_price"),
                version,
                &capitalized_price_stored,
                indexes,
                spot,
            ),
            capitalized_price_stored,
            supply_density: DensitySeries::import(
                db,
                &format!("{name}_supply_density"),
                version,
                indexes,
            )?,
        })
    }

    pub(super) fn push(&mut self, stats: &PriceStats, density: &SupplyDensity<PartsPerMillion32>) {
        self.cost_basis.push(&stats.cost_basis);
        self.capitalized_price_stored.push(stats.capitalized_price);
        self.supply_density.push(density);
    }

    pub(super) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.cost_basis
            .stored_vecs_mut()
            .chain([&mut self.capitalized_price_stored as &mut dyn AnyStoredVec])
            .chain(self.supply_density.stored_vecs_mut())
    }
}
