use bitview_traversable::Traversable;
use bitview_vecs::DailyMappings;
use brk_error::Result;
use brk_types::Version;
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::{
    AgeDensityVecs, AgePriceBoundsVecs, CostBasisData, DailyPercentilesVecs, SupplyDensityVecs,
    WeightedPair,
};

#[derive(Traversable)]
pub struct CostBasisVecs<M: StorageMode = Rw> {
    pub age_density: AgeDensityVecs<M>,
    pub age_bounds: AgePriceBoundsVecs<M>,
    pub per_coin: WeightedPair<DailyPercentilesVecs<M>>,
    pub per_dollar: WeightedPair<DailyPercentilesVecs<M>>,
    /// Daily density of each mode-weighted URPD within ±5% of closing spot. Undefined
    /// for missing snapshots, empty distributions, or non-positive spot prices.
    pub supply_density: WeightedPair<SupplyDensityVecs<M>>,
    /// Daily density within ±10% of closing spot, split at spot into profit
    /// and loss. Uses total weighted supply as denominator; missing snapshots,
    /// empty distributions, and non-positive spot prices are undefined.
    pub supply_density_10pct: WeightedPair<SupplyDensityVecs<M>>,
}

impl CostBasisVecs {
    fn import_weighting(
        db: &Database,
        weighting: &str,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<WeightedPair<DailyPercentilesVecs>> {
        WeightedPair::try_from_fn(|weight| {
            DailyPercentilesVecs::forced_import(
                db,
                &format!("bedrock_{}_cost_basis_{weighting}", weight.as_str()),
                version,
                mappings,
            )
        })
    }

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
            per_coin: Self::import_weighting(db, "per_coin", version, mappings)?,
            per_dollar: Self::import_weighting(db, "per_dollar", version, mappings)?,
            supply_density: import_density("")?,
            supply_density_10pct: import_density("_10pct")?,
        })
    }

    pub fn push(&mut self, data: &WeightedPair<CostBasisData>) {
        self.per_coin.cointime.push(&data.cointime.prices.per_coin);
        self.per_coin.coinflow.push(&data.coinflow.prices.per_coin);
        self.per_dollar
            .cointime
            .push(&data.cointime.prices.per_dollar);
        self.per_dollar
            .coinflow
            .push(&data.coinflow.prices.per_dollar);
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
        self.per_coin
            .iter_mut()
            .chain(self.per_dollar.iter_mut())
            .flat_map(DailyPercentilesVecs::collect_vecs_mut)
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
