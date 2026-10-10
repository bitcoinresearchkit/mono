use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{DensityVecs, PerBlock, PercentilesVecs, Price};
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode, WritableVec};

use super::CostBasisBlockData;

/// The stored cost-basis sources of every aggregate cohort, pushed together.
#[derive(Traversable)]
pub struct CostBasisVecs<M: StorageMode = Rw> {
    pub min: AgeAggregate<Price<PerBlock<Cents, M>>>,
    pub max: AgeAggregate<Price<PerBlock<Cents, M>>>,
    pub per_coin: AgeAggregate<PercentilesVecs<M>>,
    pub per_dollar: AgeAggregate<PercentilesVecs<M>>,
    pub supply_density: AgeAggregate<DensityVecs<M>>,
    pub capital_density: AgeAggregate<DensityVecs<M>>,
}

impl CostBasisVecs {
    /// The version of the public views over these sources.
    pub fn version(version: Version) -> Version {
        version + Version::ONE
    }

    pub fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Box<Self>> {
        let aggregate_version = Self::version(version);
        let prices = |metric: &str| {
            AgeAggregate::try_from_fn(|id| {
                Price::import(
                    db,
                    &id.metric_name(metric),
                    aggregate_version + Version::ONE,
                    mappings,
                )
            })
        };
        let percentiles = |metric: &str| {
            AgeAggregate::try_from_fn(|id| {
                PercentilesVecs::import(db, &id.metric_name(metric), version, mappings)
            })
        };
        let densities = |metric: &str| {
            AgeAggregate::try_from_fn(|id| {
                DensityVecs::import(
                    db,
                    &id.metric_name(metric),
                    aggregate_version + Version::ONE,
                    mappings,
                )
            })
        };
        Ok(Box::new(Self {
            min: prices("cost_basis_min")?,
            max: prices("cost_basis_max")?,
            per_coin: percentiles("cost_basis_per_coin")?,
            per_dollar: percentiles("cost_basis_per_dollar")?,
            supply_density: densities("supply_density")?,
            capital_density: densities("capital_density")?,
        }))
    }

    #[inline(always)]
    pub fn push(&mut self, cohort_values: AgeAggregate<CostBasisBlockData>) {
        for id in AgeAggregateId::ALL {
            let values = id.select(&cohort_values);
            id.select_mut(&mut self.min).cents.height.push(values.min);
            id.select_mut(&mut self.max).cents.height.push(values.max);
            id.select_mut(&mut self.supply_density)
                .push(&values.supply_density);
            id.select_mut(&mut self.capital_density)
                .push(&values.capital_density);
            id.select_mut(&mut self.per_coin).push(&values.per_coin);
            id.select_mut(&mut self.per_dollar).push(&values.per_dollar);
        }
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs: Vec<&mut dyn AnyStoredVec> = [&mut self.min, &mut self.max]
            .into_iter()
            .flat_map(|sources| sources.iter_mut())
            .map(|price| &mut price.cents.height as &mut dyn AnyStoredVec)
            .collect();
        vecs.extend(
            self.supply_density
                .iter_mut()
                .chain(self.capital_density.iter_mut())
                .flat_map(DensityVecs::stored_vecs_mut),
        );
        vecs.extend(
            self.per_coin
                .iter_mut()
                .chain(self.per_dollar.iter_mut())
                .flat_map(PercentilesVecs::collect_vecs_mut),
        );
        vecs
    }
}
