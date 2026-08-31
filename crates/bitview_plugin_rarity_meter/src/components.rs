use brk_error::Result;
use rayon::prelude::*;

use bitview_plugin_indexer::Indexer;
use bitview_traversable::Traversable;
use brk_exit::Exit;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use super::{Component, component};
#[derive(Traversable)]
pub struct Components<M: StorageMode = Rw> {
    /// Rarity Meter component using the all-chain realized price—the
    /// satoshi-weighted mean creation price of all unspent outputs—as its
    /// reference.
    pub realized_price: Component<M>,
    /// Rarity Meter component using the all-chain capitalized price—the mean
    /// creation price weighted by value invested at creation—as its reference.
    pub capitalized_price: Component<M>,
    /// Rarity Meter component using the satoshi-weighted mean creation price of
    /// UTXOs younger than 150 days as its reference.
    pub sth_realized_price: Component<M>,
    /// Rarity Meter component using the value-weighted mean creation price of
    /// UTXOs younger than 150 days as its reference.
    pub sth_capitalized_price: Component<M>,
    /// Rarity Meter component using the satoshi-weighted mean creation price of
    /// UTXOs at least 150 days old as its reference.
    pub lth_realized_price: Component<M>,
    /// Rarity Meter component using the value-weighted mean creation price of
    /// UTXOs at least 150 days old as its reference.
    pub lth_capitalized_price: Component<M>,
    /// Rarity Meter component using the satoshi-weighted mean creation price of
    /// UTXOs at least 180 days old as its reference.
    pub over_6m_realized_price: Component<M>,
    /// Rarity Meter component using the satoshi-weighted mean creation price of
    /// UTXOs at least 120 days old as its reference.
    pub over_4m_realized_price: Component<M>,
    /// Rarity Meter component using the satoshi-weighted mean creation price of
    /// UTXOs less than 120 days old as its reference.
    pub under_4m_realized_price: Component<M>,
    /// Rarity Meter component using the satoshi-weighted mean creation price of
    /// UTXOs less than 180 days old as its reference.
    pub under_6m_realized_price: Component<M>,
    /// Rarity Meter component using cointime vaulted price as its reference:
    /// realized price divided by one minus liveliness, where liveliness is
    /// cumulative coinblocks destroyed divided by cumulative coinblocks
    /// created.
    pub vaulted_price: Component<M>,
    /// Rarity Meter component using cointime active price as its reference:
    /// realized price divided by liveliness, where liveliness is cumulative
    /// coinblocks destroyed divided by cumulative coinblocks created.
    pub active_price: Component<M>,
    /// Rarity Meter component using cointime true market mean price as its
    /// reference: realized capitalization minus cumulative issuance-date
    /// subsidy value, divided by active supply; active supply is circulating
    /// supply multiplied by liveliness.
    pub true_market_mean_price: Component<M>,
    /// Rarity Meter component using cointime price as its reference: the
    /// cumulative sum of spot price multiplied by coinblocks destroyed, divided
    /// by cumulative coinblocks stored.
    pub cointime_price: Component<M>,
    /// Rarity Meter component using coinflow price as its reference: realized
    /// capitalization weighted by each UTXO age range's estimated eventual
    /// spending probability, divided by supply weighted by the same
    /// probability.
    pub coinflow_price: Component<M>,
}

pub fn forced_import(
    db: &Database,
    version: Version,
    mappings: &bitview_plugin_mappings::Vecs,
    distribution: &bitview_plugin_distribution::Vecs,
    cointime: &bitview_plugin_cointime::Vecs,
    coinflow: &bitview_plugin_coinflow::Vecs,
) -> Result<Components> {
    Components::forced_import(db, version, mappings, distribution, cointime, coinflow)
}

pub fn compute(
    components: &mut Components,
    indexer: &Indexer,
    distribution: &bitview_plugin_distribution::Vecs,
    cointime: &bitview_plugin_cointime::Vecs,
    coinflow: &bitview_plugin_coinflow::Vecs,
    exit: &Exit,
) -> Result<()> {
    components.compute(indexer, distribution, cointime, coinflow, exit)
}

impl Components {
    fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        distribution: &bitview_plugin_distribution::Vecs,
        cointime: &bitview_plugin_cointime::Vecs,
        coinflow: &bitview_plugin_coinflow::Vecs,
    ) -> Result<Self> {
        let utxos = &distribution.cohorts;
        let realized_price = &utxos.realized.price.cohorts;
        let capitalized_price = &utxos.realized.capitalized_price.series;

        macro_rules! import {
            ($name:expr, $source:expr) => {
                component::forced_import(db, $name, version, mappings, &$source.cents.height)?
            };
        }

        Ok(Self {
            realized_price: import!("realized_price", realized_price.all),
            capitalized_price: import!("capitalized_price", capitalized_price.all),
            sth_realized_price: import!("sth_realized_price", realized_price.term.short),
            sth_capitalized_price: import!("sth_capitalized_price", capitalized_price.sth),
            lth_realized_price: import!("lth_realized_price", realized_price.term.long),
            lth_capitalized_price: import!("lth_capitalized_price", capitalized_price.lth),
            over_6m_realized_price: import!("over_6m_realized_price", realized_price.age.over._6m),
            over_4m_realized_price: import!("over_4m_realized_price", realized_price.age.over._4m),
            under_4m_realized_price: import!(
                "under_4m_realized_price",
                realized_price.age.under._4m
            ),
            under_6m_realized_price: import!(
                "under_6m_realized_price",
                realized_price.age.under._6m
            ),
            vaulted_price: import!("vaulted_price", cointime.prices.vaulted),
            active_price: import!("active_price", cointime.prices.active),
            true_market_mean_price: import!(
                "true_market_mean_price",
                cointime.prices.true_market_mean
            ),
            cointime_price: import!("cointime_price", cointime.prices.cointime),
            coinflow_price: import!("coinflow_price", coinflow.all.price),
        })
    }

    fn compute(
        &mut self,
        indexer: &Indexer,
        distribution: &bitview_plugin_distribution::Vecs,
        cointime: &bitview_plugin_cointime::Vecs,
        coinflow: &bitview_plugin_coinflow::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();
        let utxos = &distribution.cohorts;
        let realized_price = &utxos.realized.price.cohorts;
        let capitalized_price = &utxos.realized.capitalized_price.series;

        let jobs = [
            (&mut self.realized_price, &realized_price.all.ratio.height),
            (
                &mut self.capitalized_price,
                &capitalized_price.all.ratio.height,
            ),
            (
                &mut self.sth_realized_price,
                &realized_price.term.short.ratio.height,
            ),
            (
                &mut self.sth_capitalized_price,
                &capitalized_price.sth.ratio.height,
            ),
            (
                &mut self.lth_realized_price,
                &realized_price.term.long.ratio.height,
            ),
            (
                &mut self.lth_capitalized_price,
                &capitalized_price.lth.ratio.height,
            ),
            (
                &mut self.over_6m_realized_price,
                &realized_price.age.over._6m.ratio.height,
            ),
            (
                &mut self.over_4m_realized_price,
                &realized_price.age.over._4m.ratio.height,
            ),
            (
                &mut self.under_4m_realized_price,
                &realized_price.age.under._4m.ratio.height,
            ),
            (
                &mut self.under_6m_realized_price,
                &realized_price.age.under._6m.ratio.height,
            ),
            (
                &mut self.vaulted_price,
                &cointime.prices.vaulted.ratio.height,
            ),
            (&mut self.active_price, &cointime.prices.active.ratio.height),
            (
                &mut self.true_market_mean_price,
                &cointime.prices.true_market_mean.ratio.height,
            ),
            (
                &mut self.cointime_price,
                &cointime.prices.cointime.ratio.height,
            ),
            (&mut self.coinflow_price, &coinflow.all.price.ratio.height),
        ];
        let has_work = jobs
            .iter()
            .any(|(component, source)| component.needs_compute(starting_lengths.height, *source));
        let compute =
            |(component, source)| component::compute(component, &starting_lengths, source, exit);

        if has_work {
            jobs.into_par_iter().try_for_each(compute)
        } else {
            jobs.into_iter().try_for_each(compute)
        }
    }
}
