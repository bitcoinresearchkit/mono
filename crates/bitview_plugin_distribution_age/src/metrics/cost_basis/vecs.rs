use bitview_cohort::{UTXOAggregate, UTXOAggregateId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{AggregatePercentPerBlock, PerBlock, PercentilesVecs, Price};
use brk_error::Result;
use brk_types::{Cents, PartsPerMillion32, Sats, Version};
use vecdb::{AnyStoredVec, AnyVec, Database, Rw, StorageMode, WritableVec};

use super::{CostBasis, CostBasisBlockData, CostBasisSide};
use crate::state::UnrealizedState;

#[derive(Traversable)]
pub struct CostBasisVecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    /// Cost-basis statistics for an aggregate UTXO cohort's unspent outputs at
    /// the represented block. An output's creation price is Bitcoin's spot
    /// price when that output was created.
    pub cohorts: UTXOAggregate<CostBasis>,
    #[traversable(hidden)]
    pub in_profit_per_coin_source: UTXOAggregate<Price<PerBlock<Cents, M>>>,
    #[traversable(hidden)]
    pub in_profit_per_dollar_source: UTXOAggregate<Price<PerBlock<Cents, M>>>,
    #[traversable(hidden)]
    pub in_loss_per_coin_source: UTXOAggregate<Price<PerBlock<Cents, M>>>,
    #[traversable(hidden)]
    pub in_loss_per_dollar_source: UTXOAggregate<Price<PerBlock<Cents, M>>>,
    #[traversable(hidden)]
    pub min_source: UTXOAggregate<Price<PerBlock<Cents, M>>>,
    #[traversable(hidden)]
    pub max_source: UTXOAggregate<Price<PerBlock<Cents, M>>>,
    #[traversable(hidden)]
    pub per_coin_sources: UTXOAggregate<PercentilesVecs<M>>,
    #[traversable(hidden)]
    pub per_dollar_sources: UTXOAggregate<PercentilesVecs<M>>,
    #[traversable(hidden)]
    pub supply_density_source: AggregatePercentPerBlock<PartsPerMillion32, M>,
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
        let supply_density_source = AggregatePercentPerBlock::forced_import(
            db,
            "supply_density",
            aggregate_version,
            mappings,
        )?;
        let cohorts = UTXOAggregate::from_fn(|id| CostBasis {
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
    ) -> Result<UTXOAggregate<Price<PerBlock<Cents>>>> {
        UTXOAggregate::try_from_fn(|id| {
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
    ) -> Result<UTXOAggregate<PercentilesVecs>> {
        UTXOAggregate::try_from_fn(|id| {
            let version = if matches!(id, UTXOAggregateId::All) {
                base_version + Version::ONE
            } else {
                base_version
            };
            PercentilesVecs::forced_import(db, &id.metric_name(metric), version, mappings)
        })
    }

    #[inline(always)]
    pub fn push_prices(&mut self, spot: Cents, states: &UTXOAggregate<UnrealizedState>) {
        for id in UTXOAggregateId::ALL {
            let state = id.select(states);
            id.select_mut(&mut self.in_profit_per_coin_source)
                .cents
                .height
                .push(Self::per_coin_price(spot, state, true));
            id.select_mut(&mut self.in_loss_per_coin_source)
                .cents
                .height
                .push(Self::per_coin_price(spot, state, false));
            id.select_mut(&mut self.in_profit_per_dollar_source)
                .cents
                .height
                .push(Self::per_dollar_price(spot, state, true));
            id.select_mut(&mut self.in_loss_per_dollar_source)
                .cents
                .height
                .push(Self::per_dollar_price(spot, state, false));
        }
    }

    #[inline(always)]
    fn per_coin_price(spot: Cents, state: &UnrealizedState, in_profit: bool) -> Cents {
        let (supply, unrealized) = if in_profit {
            (state.supply_in_profit, state.unrealized_profit)
        } else {
            (state.supply_in_loss, state.unrealized_loss)
        };
        let supply = supply.as_u128();
        if supply == 0 {
            return spot;
        }
        let market_value = supply * spot.as_u128() / Sats::ONE_BTC_U128;
        let invested = if in_profit {
            market_value.saturating_sub(unrealized.as_u128())
        } else {
            market_value + unrealized.as_u128()
        };
        Cents::new((invested * Sats::ONE_BTC_U128 / supply) as u64)
    }

    #[inline(always)]
    fn per_dollar_price(spot: Cents, state: &UnrealizedState, in_profit: bool) -> Cents {
        let (supply, unrealized, capitalized_cap) = if in_profit {
            (
                state.supply_in_profit,
                state.unrealized_profit,
                state.capitalized_cap_in_profit_raw,
            )
        } else {
            (
                state.supply_in_loss,
                state.unrealized_loss,
                state.capitalized_cap_in_loss_raw,
            )
        };
        let market_value = supply.as_u128() * spot.as_u128() / Sats::ONE_BTC_U128;
        let invested = if in_profit {
            market_value.saturating_sub(unrealized.as_u128())
        } else {
            market_value + unrealized.as_u128()
        };
        let invested_raw = invested * Sats::ONE_BTC_U128;
        capitalized_cap
            .checked_div(invested_raw)
            .map(|price| Cents::new(price as u64))
            .unwrap_or(spot)
    }

    #[inline(always)]
    pub fn push(&mut self, cohort_values: UTXOAggregate<CostBasisBlockData>) {
        for id in UTXOAggregateId::ALL {
            id.select_mut(&mut self.min_source)
                .cents
                .height
                .push(id.select(&cohort_values).min);
            id.select_mut(&mut self.max_source)
                .cents
                .height
                .push(id.select(&cohort_values).max);
        }
        self.supply_density_source
            .push(UTXOAggregate::from_fn(|id| {
                id.select(&cohort_values).supply_density
            }));
        self.per_coin_sources.all.push(&cohort_values.all.per_coin);
        self.per_coin_sources.sth.push(&cohort_values.sth.per_coin);
        self.per_coin_sources.lth.push(&cohort_values.lth.per_coin);
        self.per_dollar_sources
            .all
            .push(&cohort_values.all.per_dollar);
        self.per_dollar_sources
            .sth
            .push(&cohort_values.sth.per_dollar);
        self.per_dollar_sources
            .lth
            .push(&cohort_values.lth.per_dollar);
    }

    pub fn validate_computed_versions(&mut self, version: Version) -> Result<()> {
        for percentiles in self
            .per_coin_sources
            .iter_mut()
            .chain(self.per_dollar_sources.iter_mut())
        {
            percentiles.validate_computed_version_or_reset(version)?;
        }
        Ok(())
    }

    pub fn min_resume_len(&self) -> usize {
        [
            &self.in_profit_per_coin_source,
            &self.in_profit_per_dollar_source,
            &self.in_loss_per_coin_source,
            &self.in_loss_per_dollar_source,
            &self.min_source,
            &self.max_source,
        ]
        .into_iter()
        .flat_map(|sources| sources.iter())
        .map(|price| price.cents.height.len())
        .min()
        .unwrap_or_default()
        .min(self.supply_density_source.len())
        .min(
            self.per_coin_sources
                .iter()
                .chain(self.per_dollar_sources.iter())
                .map(PercentilesVecs::min_len)
                .min()
                .unwrap_or_default(),
        )
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
