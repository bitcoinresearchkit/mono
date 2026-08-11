use brk_cohort::{
    CohortContext, UTXO_AGGREGATE_FILTERS, UTXO_AGGREGATE_NAMES, UTXOAggregate, UTXOAggregateId,
};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, PartsPerMillion32, Sats, Version};
use vecdb::{AnyStoredVec, AnyVec, Database, Rw, StorageMode};

use crate::{
    distribution::state::UnrealizedState,
    indexes,
    internal::{
        ColumnarPerBlock, LazyColumnPerBlock, LazyColumnPercentPerBlock, PercentilesVecs, Price,
    },
};

use super::{CostBasis, CostBasisBlockData, CostBasisSide};

#[derive(Traversable)]
pub struct CostBasisVecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOAggregate<CostBasis>,
    #[traversable(hidden)]
    pub in_profit_per_coin_source: ColumnarPerBlock<
        Cents,
        UTXOAggregateId,
        UTXOAggregate<Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>>,
        M,
    >,
    #[traversable(hidden)]
    pub in_profit_per_dollar_source: ColumnarPerBlock<
        Cents,
        UTXOAggregateId,
        UTXOAggregate<Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>>,
        M,
    >,
    #[traversable(hidden)]
    pub in_loss_per_coin_source: ColumnarPerBlock<
        Cents,
        UTXOAggregateId,
        UTXOAggregate<Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>>,
        M,
    >,
    #[traversable(hidden)]
    pub in_loss_per_dollar_source: ColumnarPerBlock<
        Cents,
        UTXOAggregateId,
        UTXOAggregate<Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>>,
        M,
    >,
    #[traversable(hidden)]
    pub min_source: ColumnarPerBlock<
        Cents,
        UTXOAggregateId,
        UTXOAggregate<Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>>,
        M,
    >,
    #[traversable(hidden)]
    pub max_source: ColumnarPerBlock<
        Cents,
        UTXOAggregateId,
        UTXOAggregate<Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>>,
        M,
    >,
    #[traversable(hidden)]
    pub per_coin_sources: UTXOAggregate<PercentilesVecs<M>>,
    #[traversable(hidden)]
    pub per_dollar_sources: UTXOAggregate<PercentilesVecs<M>>,
    #[traversable(hidden)]
    pub supply_density_source: ColumnarPerBlock<
        PartsPerMillion32,
        UTXOAggregateId,
        UTXOAggregate<LazyColumnPercentPerBlock<PartsPerMillion32, UTXOAggregateId>>,
        M,
    >,
}

impl CostBasisVecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        let aggregate_version = version + Version::ONE;
        let in_profit_per_coin_source = Self::import_prices(
            db,
            "cost_basis_in_profit_per_coin",
            aggregate_version,
            indexes,
        )?;
        let in_profit_per_dollar_source = Self::import_prices(
            db,
            "cost_basis_in_profit_per_dollar",
            aggregate_version,
            indexes,
        )?;
        let in_loss_per_coin_source = Self::import_prices(
            db,
            "cost_basis_in_loss_per_coin",
            aggregate_version,
            indexes,
        )?;
        let in_loss_per_dollar_source = Self::import_prices(
            db,
            "cost_basis_in_loss_per_dollar",
            aggregate_version,
            indexes,
        )?;
        let min_source = Self::import_prices(db, "cost_basis_min", aggregate_version, indexes)?;
        let max_source = Self::import_prices(db, "cost_basis_max", aggregate_version, indexes)?;
        let per_coin_sources =
            Self::import_percentiles(db, "cost_basis_per_coin", version, indexes)?;
        let per_dollar_sources =
            Self::import_percentiles(db, "cost_basis_per_dollar", version, indexes)?;
        let supply_density_source = ColumnarPerBlock::forced_import(
            db,
            "supply_density_by_aggregate",
            aggregate_version,
            |source| {
                UTXOAggregate::from_fn(|id| {
                    LazyColumnPercentPerBlock::new(
                        &Self::cohort_metric_name(id, "supply_density"),
                        aggregate_version,
                        source,
                        id,
                        indexes,
                    )
                })
            },
        )?;
        let cohorts = UTXOAggregate::from_fn(|id| CostBasis {
            in_profit: CostBasisSide {
                per_coin: id.select(&in_profit_per_coin_source.series).clone(),
                per_dollar: id.select(&in_profit_per_dollar_source.series).clone(),
            },
            in_loss: CostBasisSide {
                per_coin: id.select(&in_loss_per_coin_source.series).clone(),
                per_dollar: id.select(&in_loss_per_dollar_source.series).clone(),
            },
            min: id.select(&min_source.series).clone(),
            max: id.select(&max_source.series).clone(),
            per_coin: id.select(&per_coin_sources).prices.series.clone(),
            per_dollar: id.select(&per_dollar_sources).prices.series.clone(),
            supply_density: id.select(&supply_density_source.series).clone(),
        });

        Ok(Self {
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
        })
    }

    fn import_prices(
        db: &Database,
        metric: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<
        ColumnarPerBlock<
            Cents,
            UTXOAggregateId,
            UTXOAggregate<Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>>,
        >,
    > {
        ColumnarPerBlock::forced_import(
            db,
            &format!("{metric}_cents_by_aggregate"),
            version,
            |source| {
                UTXOAggregate::from_fn(|id| {
                    Price::from_columnar_source(
                        &Self::cohort_metric_name(id, metric),
                        version,
                        source,
                        id,
                        indexes,
                    )
                })
            },
        )
    }

    fn import_percentiles(
        db: &Database,
        metric: &str,
        base_version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<UTXOAggregate<PercentilesVecs>> {
        UTXOAggregate::try_from_fn(|id| {
            let version = if matches!(id, UTXOAggregateId::All) {
                base_version + Version::ONE
            } else {
                base_version
            };
            PercentilesVecs::forced_import(
                db,
                &Self::cohort_metric_name(id, metric),
                version,
                indexes,
            )
        })
    }

    fn cohort_metric_name(id: UTXOAggregateId, metric: &str) -> String {
        CohortContext::Utxo.metric_name(
            id.select(&UTXO_AGGREGATE_FILTERS),
            id.select(&UTXO_AGGREGATE_NAMES).id,
            metric,
        )
    }

    #[inline(always)]
    pub(crate) fn push_prices(&mut self, spot: Cents, states: &UTXOAggregate<UnrealizedState>) {
        self.in_profit_per_coin_source
            .push(UTXOAggregate::from_fn(|id| {
                Self::per_coin_price(spot, id.select(states), true)
            }));
        self.in_loss_per_coin_source
            .push(UTXOAggregate::from_fn(|id| {
                Self::per_coin_price(spot, id.select(states), false)
            }));
        self.in_profit_per_dollar_source
            .push(UTXOAggregate::from_fn(|id| {
                Self::per_dollar_price(spot, id.select(states), true)
            }));
        self.in_loss_per_dollar_source
            .push(UTXOAggregate::from_fn(|id| {
                Self::per_dollar_price(spot, id.select(states), false)
            }));
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
    pub(crate) fn push(&mut self, rows: UTXOAggregate<CostBasisBlockData>) {
        self.min_source
            .push(UTXOAggregate::from_fn(|id| id.select(&rows).min));
        self.max_source
            .push(UTXOAggregate::from_fn(|id| id.select(&rows).max));
        self.supply_density_source
            .push(UTXOAggregate::from_fn(|id| id.select(&rows).supply_density));
        self.per_coin_sources.all.push(&rows.all.per_coin);
        self.per_coin_sources.sth.push(&rows.sth.per_coin);
        self.per_coin_sources.lth.push(&rows.lth.per_coin);
        self.per_dollar_sources.all.push(&rows.all.per_dollar);
        self.per_dollar_sources.sth.push(&rows.sth.per_dollar);
        self.per_dollar_sources.lth.push(&rows.lth.per_dollar);
    }

    pub(crate) fn validate_computed_versions(&mut self, version: Version) -> Result<()> {
        for percentiles in self
            .per_coin_sources
            .iter_mut()
            .chain(self.per_dollar_sources.iter_mut())
        {
            percentiles.validate_computed_version_or_reset(version)?;
        }
        Ok(())
    }

    pub(crate) fn min_len(&self) -> usize {
        self.in_profit_per_coin_source
            .height
            .len()
            .min(self.in_profit_per_dollar_source.height.len())
            .min(self.in_loss_per_coin_source.height.len())
            .min(self.in_loss_per_dollar_source.height.len())
            .min(self.min_source.height.len())
            .min(self.max_source.height.len())
            .min(self.supply_density_source.height.len())
            .min(
                self.per_coin_sources
                    .iter()
                    .chain(self.per_dollar_sources.iter())
                    .map(|percentiles| percentiles.prices.height.len())
                    .min()
                    .unwrap_or_default(),
            )
    }

    pub(crate) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = vec![
            self.in_profit_per_coin_source.stored_mut(),
            self.in_profit_per_dollar_source.stored_mut(),
            self.in_loss_per_coin_source.stored_mut(),
            self.in_loss_per_dollar_source.stored_mut(),
            self.min_source.stored_mut(),
            self.max_source.stored_mut(),
            self.supply_density_source.stored_mut(),
        ];
        vecs.extend(
            self.per_coin_sources
                .iter_mut()
                .chain(self.per_dollar_sources.iter_mut())
                .map(|percentiles| percentiles.prices.stored_mut()),
        );
        vecs
    }
}
