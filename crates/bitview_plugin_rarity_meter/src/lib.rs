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

use bitview_plugin::{
    ComputePlugin, ImportContext, Plugin, PluginGate, PluginId, PluginStorage, UpdateContext,
};
use bitview_plugin_indexer::Indexer;
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Exit, Rw, StorageMode};

use band::Band;
use component::Component;
use components::Components;
pub use dependencies::Dependencies;
use extremes::Extremes;
pub use has::HasRarityMeter;
use inner::RarityMeterInner;

use block_decay_percentiles::{BlockDecayPercentiles, START_HEIGHT};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("rarity_meter"), Version::new(16));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,

    pub components: Components<M>,
    pub extremes: Extremes<M>,
    /// Combined from ten component models: under-four-month, under-six-month,
    /// over-four-month, and over-six-month realized price; STH and LTH realized
    /// and capitalized price; and all-chain realized and capitalized price.
    pub full: RarityMeterInner<M>,
    /// Combined from four young-coin models: under-four-month and
    /// under-six-month realized price plus STH realized and capitalized price.
    pub local: RarityMeterInner<M>,
    /// Combined from six old-coin and all-chain models: over-four-month and
    /// over-six-month realized price, all-chain realized and capitalized price,
    /// and LTH realized and capitalized price.
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
        STORAGE.finalize_database(&this.db, &this)?;
        Ok(this)
    }

    fn compute_inner(
        &mut self,
        indexer: &Indexer,
        distribution: &bitview_plugin_distribution::Vecs,
        cointime: &bitview_plugin_cointime::Vecs,
        coinflow: &bitview_plugin_coinflow::Vecs,
        prices: &bitview_plugin_price::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        let spot = &prices.spot.cents.height;
        let metrics = &distribution.cohorts;
        let realized = &metrics.realized;

        components::compute(
            &mut self.components,
            indexer,
            distribution,
            cointime,
            coinflow,
            exit,
        )?;
        extremes::compute(
            &mut self.extremes,
            indexer,
            &metrics.supply.in_loss.cohorts.all.btc.height,
            &realized.profit.cohorts.all.sum._24h.usd.height,
            &realized.loss.cohorts.all.sum._24h.usd.height,
            &realized.peak_regret.series.all.sum._24h.usd.height,
            &realized.sell_side_risk_ratio.all._24h.percent.height,
            exit,
        )?;

        // Full: all Rainbow components, 10 models
        inner::compute(
            &mut self.full,
            &[
                &self.components.under_4m_realized_price,
                &self.components.under_6m_realized_price,
                &self.components.over_4m_realized_price,
                &self.components.over_6m_realized_price,
                &self.components.sth_realized_price,
                &self.components.sth_capitalized_price,
                &self.components.lth_realized_price,
                &self.components.lth_capitalized_price,
                &self.components.realized_price,
                &self.components.capitalized_price,
            ],
            spot,
            indexer,
            exit,
        )?;

        // Local: young-coin and STH components, 4 models
        inner::compute(
            &mut self.local,
            &[
                &self.components.under_4m_realized_price,
                &self.components.under_6m_realized_price,
                &self.components.sth_realized_price,
                &self.components.sth_capitalized_price,
            ],
            spot,
            indexer,
            exit,
        )?;

        // Cycle: old-coin, all, and LTH components, 6 models
        inner::compute(
            &mut self.cycle,
            &[
                &self.components.over_4m_realized_price,
                &self.components.over_6m_realized_price,
                &self.components.realized_price,
                &self.components.capitalized_price,
                &self.components.lth_realized_price,
                &self.components.lth_capitalized_price,
            ],
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
        self.compute_inner(
            dependencies.indexer,
            dependencies.distribution,
            dependencies.cointime,
            dependencies.coinflow,
            dependencies.price,
            context.exit(),
        )
    }
}
