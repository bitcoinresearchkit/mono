use crate::density_sources::DensitySources;
use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{PerBlock, PercentilesVecs, Price};
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode, WritableVec};

use super::{CostBasis, CostBasisBlockData, CostBasisSide};
use crate::unrealized_data::UnrealizedData;

#[derive(Traversable)]
pub struct CostBasisVecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    /// Cost-basis statistics for an aggregate UTXO cohort's unspent outputs at
    /// the represented block. An output's creation price is Bitcoin's spot
    /// price when that output was created.
    pub cohorts: AgeAggregate<CostBasis>,
    #[traversable(hidden)]
    pub in_profit_per_coin_source: AgeAggregate<Price<PerBlock<Cents, M>>>,
    #[traversable(hidden)]
    pub in_profit_per_dollar_source: AgeAggregate<Price<PerBlock<Cents, M>>>,
    #[traversable(hidden)]
    pub in_loss_per_coin_source: AgeAggregate<Price<PerBlock<Cents, M>>>,
    #[traversable(hidden)]
    pub in_loss_per_dollar_source: AgeAggregate<Price<PerBlock<Cents, M>>>,
    #[traversable(hidden)]
    pub min_source: AgeAggregate<Price<PerBlock<Cents, M>>>,
    #[traversable(hidden)]
    pub max_source: AgeAggregate<Price<PerBlock<Cents, M>>>,
    #[traversable(hidden)]
    pub per_coin_sources: AgeAggregate<PercentilesVecs<M>>,
    #[traversable(hidden)]
    pub per_dollar_sources: AgeAggregate<PercentilesVecs<M>>,
    #[traversable(hidden)]
    supply_density_source: DensitySources<M>,
}

impl CostBasisVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
    ) -> Result<Box<Self>> {
        let aggregate_version = version + Version::ONE;
        let in_profit_per_coin_source = Self::import_prices(
            db,
            "cost_basis_in_profit_per_coin",
            aggregate_version,
            mappings,
        )?;
        let in_profit_per_dollar_source = Self::import_prices(
            db,
            "cost_basis_in_profit_per_dollar",
            aggregate_version,
            mappings,
        )?;
        let in_loss_per_coin_source = Self::import_prices(
            db,
            "cost_basis_in_loss_per_coin",
            aggregate_version,
            mappings,
        )?;
        let in_loss_per_dollar_source = Self::import_prices(
            db,
            "cost_basis_in_loss_per_dollar",
            aggregate_version,
            mappings,
        )?;
        let min_source = Self::import_prices(db, "cost_basis_min", aggregate_version, mappings)?;
        let max_source = Self::import_prices(db, "cost_basis_max", aggregate_version, mappings)?;
        let per_coin_sources =
            Self::import_percentiles(db, "cost_basis_per_coin", version, mappings)?;
        let per_dollar_sources =
            Self::import_percentiles(db, "cost_basis_per_dollar", version, mappings)?;
        let supply_density_source =
            DensitySources::forced_import(db, "supply_density", aggregate_version, mappings)?;
        let cohorts = AgeAggregate::from_fn(|id| CostBasis {
            in_profit: CostBasisSide {
                per_coin: Price::from_height_source(
                    &id.metric_name("cost_basis_in_profit_per_coin"),
                    aggregate_version,
                    &id.select(&in_profit_per_coin_source).cents.height,
                    mappings,
                ),
                per_dollar: Price::from_height_source(
                    &id.metric_name("cost_basis_in_profit_per_dollar"),
                    aggregate_version,
                    &id.select(&in_profit_per_dollar_source).cents.height,
                    mappings,
                ),
            },
            in_loss: CostBasisSide {
                per_coin: Price::from_height_source(
                    &id.metric_name("cost_basis_in_loss_per_coin"),
                    aggregate_version,
                    &id.select(&in_loss_per_coin_source).cents.height,
                    mappings,
                ),
                per_dollar: Price::from_height_source(
                    &id.metric_name("cost_basis_in_loss_per_dollar"),
                    aggregate_version,
                    &id.select(&in_loss_per_dollar_source).cents.height,
                    mappings,
                ),
            },
            min: Price::from_height_source(
                &id.metric_name("cost_basis_min"),
                aggregate_version,
                &id.select(&min_source).cents.height,
                mappings,
            ),
            max: Price::from_height_source(
                &id.metric_name("cost_basis_max"),
                aggregate_version,
                &id.select(&max_source).cents.height,
                mappings,
            ),
            per_coin: id.select(&per_coin_sources).prices.clone(),
            per_dollar: id.select(&per_dollar_sources).prices.clone(),
            supply_density: id.select(&supply_density_source.series).clone(),
        });

        Ok(Box::new(Self {
            cohorts,
            in_profit_per_coin_source,
            in_profit_per_dollar_source,
            in_loss_per_coin_source,
            in_loss_per_dollar_source,
            min_source,
            max_source,
            per_coin_sources,
            per_dollar_sources,
            supply_density_source,
        }))
    }

    fn import_prices(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
    ) -> Result<AgeAggregate<Price<PerBlock<Cents>>>> {
        AgeAggregate::try_from_fn(|id| {
            Price::forced_import(
                db,
                &id.metric_name(metric),
                version + Version::ONE,
                mappings,
            )
        })
    }

    fn import_percentiles(
        db: &Database,
        metric: &str,
        base_version: Version,
        mappings: &MappingsVecs,
    ) -> Result<AgeAggregate<PercentilesVecs>> {
        AgeAggregate::try_from_fn(|id| {
            let version = base_version;
            PercentilesVecs::forced_import(db, &id.metric_name(metric), version, mappings)
        })
    }

    #[inline(always)]
    pub(crate) fn push_prices(&mut self, data: &AgeAggregate<UnrealizedData>) {
        for id in AgeAggregateId::ALL {
            let d = id.select(data);
            id.select_mut(&mut self.in_profit_per_coin_source)
                .cents
                .height
                .push(d.profit_per_coin);
            id.select_mut(&mut self.in_loss_per_coin_source)
                .cents
                .height
                .push(d.loss_per_coin);
            id.select_mut(&mut self.in_profit_per_dollar_source)
                .cents
                .height
                .push(d.profit_per_dollar);
            id.select_mut(&mut self.in_loss_per_dollar_source)
                .cents
                .height
                .push(d.loss_per_dollar);
        }
    }

    #[inline(always)]
    pub fn push(&mut self, cohort_values: AgeAggregate<CostBasisBlockData>) {
        for id in AgeAggregateId::ALL {
            id.select_mut(&mut self.min_source)
                .cents
                .height
                .push(id.select(&cohort_values).min);
            id.select_mut(&mut self.max_source)
                .cents
                .height
                .push(id.select(&cohort_values).max);
        }
        self.supply_density_source.push(AgeAggregate::from_fn(|id| {
            id.select(&cohort_values).supply_density
        }));
        for id in AgeAggregateId::ALL {
            id.select_mut(&mut self.per_coin_sources)
                .push(&id.select(&cohort_values).per_coin);
            id.select_mut(&mut self.per_dollar_sources)
                .push(&id.select(&cohort_values).per_dollar);
        }
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs: Vec<&mut dyn AnyStoredVec> = [
            &mut self.in_profit_per_coin_source,
            &mut self.in_profit_per_dollar_source,
            &mut self.in_loss_per_coin_source,
            &mut self.in_loss_per_dollar_source,
            &mut self.min_source,
            &mut self.max_source,
        ]
        .into_iter()
        .flat_map(|sources| sources.iter_mut())
        .map(|price| &mut price.cents.height as &mut dyn AnyStoredVec)
        .collect();
        vecs.extend(self.supply_density_source.collect_vecs_mut());
        vecs.extend(
            self.per_coin_sources
                .iter_mut()
                .chain(self.per_dollar_sources.iter_mut())
                .flat_map(PercentilesVecs::collect_vecs_mut),
        );
        vecs
    }
}
