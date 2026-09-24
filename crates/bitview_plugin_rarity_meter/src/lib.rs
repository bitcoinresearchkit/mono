#![allow(clippy::type_complexity)]

use band::Band;
use bitview_plugin::{
    ComputePlugin, ImportContext, Plugin, PluginId, PluginStorage, UpdateContext,
};
use bitview_plugin_coinflow::Vecs as CoinflowVecs;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_plugin_distribution::Vecs as DistributionVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{DailyView, RepeatDay};
use block_decay_percentiles::{BlockDecayPercentiles, START_HEIGHT};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use component::Component;
use components::Components;
use extremes::Extremes;
use inner::RarityMeterInner;
use rayon::{join, prelude::*};
use reference_prices::ReferencePrices;
use vecdb::{Database, Rw, StorageMode};

mod band;
mod block_decay_percentiles;
mod component;
mod component_price;
mod components;
mod dependencies;
mod extreme;
mod extremes;
mod has;
mod inner;
mod median_component;
#[cfg(test)]
mod recovery_tests;
mod reference_price;
mod reference_prices;
#[cfg(test)]
#[path = "../../bitview_vecs/tests/common/mod.rs"]
mod test_common;
mod threshold_vecs;

pub use dependencies::Dependencies;

pub use has::HasRarityMeter;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("rarity_meter"), Version::new(17));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Model-specific realized and capitalized prices reconstructed from the
    /// distribution's disjoint raw capitalization and supply histories.
    pub reference_prices: ReferencePrices<M>,

    /// Reference-price components used by the Rarity Meter. A UTXO's creation
    /// price is Bitcoin's spot price when that output was created. Realized
    /// price is `sum(creation price x unspent sats) / sum(unspent sats)`;
    /// capitalized price instead weights by creation-date value and is
    /// `sum(creation price squared x unspent sats) / sum(creation price x
    /// unspent sats)`.
    pub components: Components<M>,
    pub extremes: Extremes<M>,
    /// Full Rarity Meter combining local and cycle views to show how unusual
    /// spot price is across both young-coin and long-cycle reference models.
    pub full: RarityMeterInner<M>,
    /// Full V2 combines 24 reference-price components, excluding LTH realized,
    /// capitalized, and median prices, with the three lower-only Bedrock models.
    pub full_v2: RarityMeterInner<M>,
    /// Local Rarity Meter focused on young-coin positioning. It combines
    /// under-four-month and under-six-month realized price with short-term-holder
    /// realized and capitalized price.
    pub local: RarityMeterInner<M>,
    /// Local V2 adds under-four-month and under-six-month capitalized prices
    /// and BTC- and USD-weighted STH median prices to Local's four reference models.
    pub local_v2: RarityMeterInner<M>,
    /// Cycle Rarity Meter focused on long-cycle valuation. It combines six
    /// old-coin and all-chain reference-price models with rare lower-price
    /// boundaries from the raw, cointime, and coinflow Bedrock models.
    pub cycle: RarityMeterInner<M>,
    /// Cycle V2 combines 16 all-chain, older-coin, cointime, and coinflow
    /// reference prices with three Bedrock floors, excluding LTH-specific prices.
    pub cycle_v2: RarityMeterInner<M>,
}

const COMPUTE_BATCH_SIZE: usize = 100_000;

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        distribution: &DistributionVecs,
        cointime: &CointimeVecs,
        coinflow: &CoinflowVecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let version = STORAGE.schema_version();
        let reference_prices = ReferencePrices::forced_import(&db, version, mappings)?;
        let this = Self {
            components: components::forced_import(
                &db,
                version,
                mappings,
                distribution,
                &reference_prices,
                cointime,
                coinflow,
            )?,
            reference_prices,
            extremes: extremes::forced_import(&db, version, mappings)?,
            full: inner::forced_import(&db, "rarity_meter", version, mappings)?,
            full_v2: inner::forced_import(&db, "rarity_meter_v2", version, mappings)?,
            local: inner::forced_import(&db, "local_rarity_meter", version, mappings)?,
            local_v2: inner::forced_import(&db, "local_rarity_meter_v2", version, mappings)?,
            cycle: inner::forced_import(&db, "cycle_rarity_meter", version, mappings)?,
            cycle_v2: inner::forced_import(&db, "cycle_rarity_meter_v2", version, mappings)?,
            db,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<Self::Output> {
        let exit = context.exit();
        let Dependencies {
            indexer,
            bedrock,
            distribution,
            cointime,
            coinflow,
            price: prices,
        } = dependencies;
        self.db.sync_bg_tasks()?;

        let spot = &prices.spot.cents.height;
        let metrics = &distribution.cohorts;
        let realized = &metrics.realized;
        let cap_raw = &realized.cap_raw;
        let capitalized_cap_raw = &realized.capitalized_cap_raw;
        let supply = &metrics.supply.total.cohorts.utxo;

        self.reference_prices.compute(
            indexer.safe_lengths().height,
            [
                &cap_raw.term.short,
                &cap_raw.term.long,
                &cap_raw.age._4m_to_5m,
                &cap_raw.age._5m_to_6m,
            ],
            [
                &supply.term.short.sats.height,
                &supply.term.long.sats.height,
                &supply.age._4m_to_5m.sats.height,
                &supply.age._5m_to_6m.sats.height,
            ],
            [
                &capitalized_cap_raw.term.short,
                &capitalized_cap_raw.age._4m_to_5m,
                &capitalized_cap_raw.age._5m_to_6m,
            ],
            spot,
            exit,
        )?;

        let (components_result, extremes_result) = join(
            || {
                components::compute(
                    &mut self.components,
                    indexer,
                    distribution,
                    &self.reference_prices,
                    cointime,
                    coinflow,
                    spot,
                    exit,
                )
            },
            || {
                extremes::compute(
                    &mut self.extremes,
                    indexer,
                    &metrics.supply.in_loss.cohorts.all.btc.height,
                    &realized.profit.cohorts.utxo.all.sum._24h.usd.height,
                    &realized.loss.cohorts.utxo.all.sum._24h.usd.height,
                    &realized.peak_regret.series.all.sum._24h.usd.height,
                    &realized.sell_side_risk_ratio.all._24h.percent.height,
                    exit,
                )
            },
        );
        components_result?;
        extremes_result?;

        let local_components = [
            &self.components.under_4m_realized_price,
            &self.components.under_6m_realized_price,
            &self.components.sth_realized_price,
            &self.components.sth_capitalized_price,
        ];

        // Bedrock floors run from the rarest low boundary to the broadest one,
        // matching the rarity meter's P0.1, P0.5, P1, P2, and P5 order.
        let bedrock_floors = [
            [
                &bedrock.raw.floor.pct99_9.cents.views.height,
                &bedrock.raw.floor.pct99_5.cents.views.height,
                &bedrock.raw.floor.pct99.cents.views.height,
                &bedrock.raw.floor.pct98.cents.views.height,
                &bedrock.raw.floor.pct95.cents.views.height,
            ],
            [
                &bedrock.cointime.floor.pct99_9.cents.views.height,
                &bedrock.cointime.floor.pct99_5.cents.views.height,
                &bedrock.cointime.floor.pct99.cents.views.height,
                &bedrock.cointime.floor.pct98.cents.views.height,
                &bedrock.cointime.floor.pct95.cents.views.height,
            ],
            [
                &bedrock.coinflow.floor.pct99_9.cents.views.height,
                &bedrock.coinflow.floor.pct99_5.cents.views.height,
                &bedrock.coinflow.floor.pct99.cents.views.height,
                &bedrock.coinflow.floor.pct98.cents.views.height,
                &bedrock.coinflow.floor.pct95.cents.views.height,
            ],
        ];

        let cycle_components = [
            &self.components.over_4m_realized_price,
            &self.components.over_6m_realized_price,
            &self.components.realized_price,
            &self.components.capitalized_price,
            &self.components.lth_realized_price,
            &self.components.lth_capitalized_price,
        ];
        let local_v2_components = [
            &self.components.under_4m_realized_price,
            &self.components.under_6m_realized_price,
            &self.components.under_4m_capitalized_price,
            &self.components.under_6m_capitalized_price,
            &self.components.sth_realized_price,
            &self.components.sth_capitalized_price,
            &self.components.sth_median_price_btc_weighted.component,
            &self.components.sth_median_price_usd_weighted.component,
        ];
        let cycle_v2_components = [
            &self.components.realized_price,
            &self.components.capitalized_price,
            &self.components.median_price_btc_weighted.component,
            &self.components.median_price_usd_weighted.component,
            &self.components.cointime_median_price_btc_weighted.component,
            &self.components.cointime_median_price_usd_weighted.component,
            &self.components.coinflow_median_price_btc_weighted.component,
            &self.components.coinflow_median_price_usd_weighted.component,
            &self.components.over_6m_realized_price,
            &self.components.over_4m_realized_price,
            &self.components.vaulted_price,
            &self.components.active_price,
            &self.components.true_market_mean_price,
            &self.components.cointime_price,
            &self.components.awake_price,
            &self.components.coinflow_price,
        ];
        let starting_height = indexer.safe_lengths().height;
        // Daily median revisions affect every block of their day, including
        // blocks before the indexer's safe resume height.
        let v2_starting_height = self
            .components
            .cointime_median_price_btc_weighted
            .starting_height(starting_height);
        let jobs: [(
            &mut RarityMeterInner,
            &[&Component],
            &[[&DailyView<Height, Cents, RepeatDay>; 5]],
            Height,
        ); 4] = [
            (&mut self.local, &local_components, &[], starting_height),
            (
                &mut self.cycle,
                &cycle_components,
                &bedrock_floors,
                starting_height,
            ),
            (
                &mut self.local_v2,
                &local_v2_components,
                &[],
                starting_height,
            ),
            (
                &mut self.cycle_v2,
                &cycle_v2_components,
                &bedrock_floors,
                v2_starting_height,
            ),
        ];
        let has_work = jobs
            .iter()
            .any(|(inner, components, lower_components, start)| {
                inner.needs_compute(components, lower_components, spot, *start)
            });
        let compute = |(inner, components, lower_components, start)| {
            inner::compute(inner, components, lower_components, spot, start, exit)
        };

        if has_work {
            jobs.into_par_iter().try_for_each(compute)?;
        } else {
            jobs.into_iter().try_for_each(compute)?;
        }

        // Full inherits every boundary and score from Local and Cycle.
        inner::compute_combined(
            &mut self.full,
            &[&self.local, &self.cycle],
            spot,
            starting_height,
            exit,
        )?;

        inner::compute_combined(
            &mut self.full_v2,
            &[&self.local_v2, &self.cycle_v2],
            spot,
            v2_starting_height,
            exit,
        )?;

        context.compact_database(&self.db);

        Ok(())
    }
}

#[cfg(test)]
#[allow(dead_code)]
#[path = "../tests/common/cache.rs"]
mod test_cache;
