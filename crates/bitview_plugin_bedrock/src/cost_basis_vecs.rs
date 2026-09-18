use bitview_cohort::UTXOAggregate;
use bitview_traversable::Traversable;
use bitview_vecs::DailyMappings;
use brk_error::Result;
use brk_types::Version;
use derive_more::Deref;
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::{
    AgeDensityVecs, AgePriceBoundsVecs, CostBasisData, CostBasisDistributionVecs,
    SupplyDensityVecs, WeightedPair,
};

#[derive(Deref, Traversable)]
pub struct CostBasisVecs<M: StorageMode = Rw> {
    pub age_density: AgeDensityVecs<M>,
    pub age_bounds: AgePriceBoundsVecs<M>,
    pub sth: CostBasisDistributionVecs<M>,
    pub lth: CostBasisDistributionVecs<M>,
    #[deref]
    #[traversable(flatten)]
    pub distribution: CostBasisDistributionVecs<M>,
    /// Daily density of each mode-weighted URPD within ±5% of closing spot. Undefined
    /// for missing snapshots, empty distributions, or non-positive spot prices.
    pub supply_density: WeightedPair<SupplyDensityVecs<M>>,
    /// Daily density within ±10% of closing spot, split at spot into profit
    /// and loss. Uses total weighted supply as denominator; missing snapshots,
    /// empty distributions, and non-positive spot prices are undefined.
    pub supply_density_10pct: WeightedPair<SupplyDensityVecs<M>>,
}

impl CostBasisVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<Self> {
        let import_density = |suffix| {
            WeightedPair::try_from_fn(|weight| {
                SupplyDensityVecs::forced_import(
                    db,
                    &format!("bedrock_{}_supply_density{suffix}", weight.as_str()),
                    version,
                    mappings,
                )
            })
        };
        Ok(Self {
            age_density: AgeDensityVecs::forced_import(db, version, mappings)?,
            age_bounds: AgePriceBoundsVecs::forced_import(db, version, mappings)?,
            sth: CostBasisDistributionVecs::forced_import(db, "_sth", version, mappings)?,
            lth: CostBasisDistributionVecs::forced_import(db, "_lth", version, mappings)?,
            distribution: CostBasisDistributionVecs::forced_import(db, "", version, mappings)?,
            supply_density: import_density("")?,
            supply_density_10pct: import_density("_10pct")?,
        })
    }

    pub fn push(&mut self, data: &UTXOAggregate<WeightedPair<CostBasisData>>) {
        self.sth.push(&data.sth);
        self.lth.push(&data.lth);
        let data = &data.all;
        self.distribution.push(data);
        self.supply_density
            .cointime
            .push(&data.cointime.supply_density);
        self.supply_density
            .coinflow
            .push(&data.coinflow.supply_density);
        self.supply_density_10pct
            .cointime
            .push(&data.cointime.supply_density_10pct);
        self.supply_density_10pct
            .coinflow
            .push(&data.coinflow.supply_density_10pct);
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.distribution
            .stored_vecs_mut()
            .chain(self.sth.stored_vecs_mut())
            .chain(self.lth.stored_vecs_mut())
            .chain(
                self.supply_density
                    .iter_mut()
                    .chain(self.supply_density_10pct.iter_mut())
                    .flat_map(SupplyDensityVecs::stored_vecs_mut),
            )
    }

    pub fn minimum_len(&mut self) -> usize {
        self.stored_vecs_mut()
            .map(|vec| vec.len())
            .min()
            .unwrap_or_default()
    }
}

#[cfg(test)]
#[path = "../tests/unit/cost_basis_vecs.rs"]
mod tests;
