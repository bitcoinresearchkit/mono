#![allow(clippy::type_complexity)]

mod band;
mod block_decay_percentiles;
mod cached_component_price;
mod component;
mod components;
mod dependencies;
mod extreme;
mod extremes;
mod has;
mod inner;
mod percentiles;
mod threshold_vecs;

use brk_error::Result;
use rayon::{join, prelude::*};

use bitview_compute::{DailyView, RepeatDay};
use bitview_plugin::{
    ComputePlugin, ImportContext, Plugin, PluginGate, PluginId, PluginStorage, UpdateContext,
};
use bitview_traversable::Traversable;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, Rw, StorageMode};

use band::Band;
use component::Component;
use components::Components;
pub use dependencies::Dependencies;
use extremes::Extremes;
pub use has::HasRarityMeter;
use inner::RarityMeterInner;

use block_decay_percentiles::{BlockDecayPercentiles, START_HEIGHT};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("rarity_meter"), Version::new(17));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,

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
    /// Local Rarity Meter focused on young-coin positioning. It combines
    /// under-four-month and under-six-month realized price with short-term-holder
    /// realized and capitalized price.
    pub local: RarityMeterInner<M>,
    /// Cycle Rarity Meter focused on long-cycle valuation. It combines six
    /// old-coin and all-chain reference-price models with rare lower-price
    /// boundaries from the raw, cointime, and coinflow Bedrock models.
    pub cycle: RarityMeterInner<M>,
}

const COMPUTE_BATCH_SIZE: usize = 100_000;

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &bitview_plugin_mappings::Vecs,
        distribution: &bitview_plugin_distribution::Vecs,
        cointime: &bitview_plugin_cointime::Vecs,
        coinflow: &bitview_plugin_coinflow::Vecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let version = STORAGE.schema_version();
        let this = Self {
            plugin_gate: Default::default(),
            components: components::forced_import(
                &db,
                version,
                mappings,
                distribution,
                cointime,
                coinflow,
            )?,
            extremes: extremes::forced_import(&db, version, mappings)?,
            full: inner::forced_import(&db, "rarity_meter", version, mappings)?,
            local: inner::forced_import(&db, "local_rarity_meter", version, mappings)?,
            cycle: inner::forced_import(&db, "cycle_rarity_meter", version, mappings)?,
            db,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }

    fn compute_inner(&mut self, dependencies: Dependencies<'_>, exit: &Exit) -> Result<()> {
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

        let (components_result, extremes_result) = join(
            || {
                components::compute(
                    &mut self.components,
                    indexer,
                    distribution,
                    cointime,
                    coinflow,
                    exit,
                )
            },
            || {
                extremes::compute(
                    &mut self.extremes,
                    indexer,
                    &metrics.supply.in_loss.cohorts.all.btc.height,
                    &realized.profit.cohorts.all.sum._24h.usd.height,
                    &realized.loss.cohorts.all.sum._24h.usd.height,
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
        let jobs: [(
            &mut RarityMeterInner,
            &[&Component],
            &[[&DailyView<Height, Cents, RepeatDay>; 5]],
        ); 2] = [
            (&mut self.local, &local_components, &[]),
            (&mut self.cycle, &cycle_components, &bedrock_floors),
        ];
        let starting_height = indexer.safe_lengths().height;
        let has_work = jobs.iter().any(|(inner, components, lower_components)| {
            inner.needs_compute(components, lower_components, spot, starting_height)
        });
        let compute = |(inner, components, lower_components)| {
            inner::compute(inner, components, lower_components, spot, indexer, exit)
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
            indexer,
            exit,
        )?;

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });

        Ok(())
    }
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
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
        self.compute_inner(dependencies, context.exit())
    }
}
