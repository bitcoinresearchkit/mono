use bitview_cohort::AgeAggregateId;
use bitview_primitives::{CostBasisByPercentile, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::IndexSources;
use brk_error::Result;
use brk_types::Version;
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use super::{
    cost_basis::CostBasisMetrics,
    density::{DensitySeries, SupplyDensity},
};

/// The same stored and derived metrics for every URPD age filter.
#[derive(Traversable)]
pub struct CohortMetrics<M: StorageMode = Rw> {
    /// Coin- and capital-weighted percentiles of rounded creation prices.
    pub cost_basis: CostBasisMetrics<M>,
    /// Weighted supply within 5% of closing spot, divided by this cohort's supply.
    #[traversable(wrap = "cost_basis", rename = "supply_density")]
    pub supply_density: DensitySeries<M>,
}

impl CohortMetrics {
    pub fn import(
        db: &Database,
        owner: &str,
        cohort: AgeAggregateId,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self {
            cost_basis: CostBasisMetrics::import(
                db,
                &cohort.metric_name(&format!("{owner}_cost_basis")),
                version,
                indexes,
            )?,
            supply_density: DensitySeries::import(
                db,
                &cohort.metric_name(&format!("{owner}_supply_density")),
                version,
                indexes,
            )?,
        })
    }

    pub(super) fn push(
        &mut self,
        cost_basis: &CostBasisByPercentile,
        density: &SupplyDensity<PartsPerMillion32>,
    ) {
        self.cost_basis.push(cost_basis);
        self.supply_density.push(density);
    }

    pub(super) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.cost_basis
            .stored_vecs_mut()
            .chain(self.supply_density.stored_vecs_mut())
    }
}
