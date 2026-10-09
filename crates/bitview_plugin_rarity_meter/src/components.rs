use bitview_plugin_coinflow::Vecs as CoinflowVecs;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use rayon::prelude::*;
use vecdb::{Database, ReadableVec, Rw, StorageMode};

use super::{Component, median_component::MedianComponent};

#[derive(Traversable)]
pub struct Components<M: StorageMode = Rw> {
    /// Rarity Meter component using the all-chain realized price—the
    /// satoshi-weighted mean creation price of all unspent outputs—as its
    /// reference.
    pub realized_price: Component<M>,
    /// Rarity Meter component using the all-chain capitalized price—the mean
    /// creation price weighted by value invested at creation—as its reference.
    pub capitalized_price: Component<M>,
    /// Rarity Meter component using the median creation price,
    /// weighted by BTC supply.
    pub median_price_btc_weighted: MedianComponent<M>,
    /// Rarity Meter component using the median creation price,
    /// weighted by creation-date USD value.
    pub median_price_usd_weighted: MedianComponent<M>,
    /// Rarity Meter component using the median creation price of UTXOs
    /// younger than 150 days, weighted by BTC supply.
    pub sth_median_price_btc_weighted: MedianComponent<M>,
    /// Rarity Meter component using the median creation price of UTXOs
    /// younger than 150 days, weighted by creation-date USD value.
    pub sth_median_price_usd_weighted: MedianComponent<M>,
    /// Rarity Meter component using the median creation price of UTXOs
    /// at least 150 days old, weighted by BTC supply.
    pub lth_median_price_btc_weighted: MedianComponent<M>,
    /// Rarity Meter component using the median creation price of UTXOs
    /// at least 150 days old, weighted by creation-date USD value.
    pub lth_median_price_usd_weighted: MedianComponent<M>,
    /// Rarity Meter component using the cointime-weighted median creation price,
    /// weighted by BTC supply.
    pub cointime_median_price_btc_weighted: MedianComponent<M>,
    /// Rarity Meter component using the cointime-weighted median creation price,
    /// weighted by creation-date USD value.
    pub cointime_median_price_usd_weighted: MedianComponent<M>,
    /// Rarity Meter component using the coinflow-weighted median creation price,
    /// weighted by BTC supply.
    pub coinflow_median_price_btc_weighted: MedianComponent<M>,
    /// Rarity Meter component using the coinflow-weighted median creation price,
    /// weighted by creation-date USD value.
    pub coinflow_median_price_usd_weighted: MedianComponent<M>,
    /// STH cointime-weighted median creation price, per coin.
    pub sth_cointime_median_price_btc_weighted: MedianComponent<M>,
    /// STH cointime-weighted median creation price, per dollar.
    pub sth_cointime_median_price_usd_weighted: MedianComponent<M>,
    /// LTH cointime-weighted median creation price, per coin.
    pub lth_cointime_median_price_btc_weighted: MedianComponent<M>,
    /// LTH cointime-weighted median creation price, per dollar.
    pub lth_cointime_median_price_usd_weighted: MedianComponent<M>,
    /// STH coinflow-weighted median creation price, per coin.
    pub sth_coinflow_median_price_btc_weighted: MedianComponent<M>,
    /// STH coinflow-weighted median creation price, per dollar.
    pub sth_coinflow_median_price_usd_weighted: MedianComponent<M>,
    /// LTH coinflow-weighted median creation price, per coin.
    pub lth_coinflow_median_price_btc_weighted: MedianComponent<M>,
    /// LTH coinflow-weighted median creation price, per dollar.
    pub lth_coinflow_median_price_usd_weighted: MedianComponent<M>,
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
    /// Rarity Meter component using the value-weighted mean creation price of
    /// UTXOs less than 120 days old as its reference.
    pub under_4m_capitalized_price: Component<M>,
    /// Rarity Meter component using the value-weighted mean creation price of
    /// UTXOs less than 180 days old as its reference.
    pub under_6m_capitalized_price: Component<M>,
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
    /// Rarity Meter component using wakefulness-weighted mean creation price.
    pub awake_price: Component<M>,
    /// Rarity Meter component using coinflow price as its reference: realized
    /// capitalization weighted by each UTXO age range's estimated eventual
    /// spending probability, divided by supply weighted by the same
    /// probability.
    pub coinflow_price: Component<M>,
}

impl Components {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        holders: &HoldersVecs,
        cointime: &CointimeVecs,
        coinflow: &CoinflowVecs,
    ) -> Result<Self> {
        let utxos = &holders.cohorts;

        macro_rules! import {
            ($name:expr, $source:expr) => {
                Component::import(db, $name, version, mappings, &$source.cents.height)?
            };
        }

        Ok(Components {
            realized_price: import!("realized_price", utxos.all.realized.price),
            capitalized_price: import!("capitalized_price", utxos.all.realized.capitalized_price),
            median_price_btc_weighted: MedianComponent::import(
                db,
                "median_price_btc_weighted",
                version,
                mappings,
                &utxos.all.cost_basis.per_coin.pct50.cents.height,
            )?,
            median_price_usd_weighted: MedianComponent::import(
                db,
                "median_price_usd_weighted",
                version,
                mappings,
                &utxos.all.cost_basis.per_dollar.pct50.cents.height,
            )?,
            sth_median_price_btc_weighted: MedianComponent::import(
                db,
                "sth_median_price_btc_weighted",
                version,
                mappings,
                &utxos.sth.cost_basis.per_coin.pct50.cents.height,
            )?,
            sth_median_price_usd_weighted: MedianComponent::import(
                db,
                "sth_median_price_usd_weighted",
                version,
                mappings,
                &utxos.sth.cost_basis.per_dollar.pct50.cents.height,
            )?,
            lth_median_price_btc_weighted: MedianComponent::import(
                db,
                "lth_median_price_btc_weighted",
                version,
                mappings,
                &utxos.lth.cost_basis.per_coin.pct50.cents.height,
            )?,
            lth_median_price_usd_weighted: MedianComponent::import(
                db,
                "lth_median_price_usd_weighted",
                version,
                mappings,
                &utxos.lth.cost_basis.per_dollar.pct50.cents.height,
            )?,
            cointime_median_price_btc_weighted: MedianComponent::import(
                db,
                "cointime_median_price_btc_weighted",
                version,
                mappings,
                &cointime
                    .urpd
                    .cohorts
                    .all
                    .cost_basis
                    .per_coin
                    .pct50
                    .cents
                    .height,
            )?,
            cointime_median_price_usd_weighted: MedianComponent::import(
                db,
                "cointime_median_price_usd_weighted",
                version,
                mappings,
                &cointime
                    .urpd
                    .cohorts
                    .all
                    .cost_basis
                    .per_dollar
                    .pct50
                    .cents
                    .height,
            )?,
            coinflow_median_price_btc_weighted: MedianComponent::import(
                db,
                "coinflow_median_price_btc_weighted",
                version,
                mappings,
                &coinflow
                    .urpd
                    .cohorts
                    .all
                    .cost_basis
                    .per_coin
                    .pct50
                    .cents
                    .height,
            )?,
            coinflow_median_price_usd_weighted: MedianComponent::import(
                db,
                "coinflow_median_price_usd_weighted",
                version,
                mappings,
                &coinflow
                    .urpd
                    .cohorts
                    .all
                    .cost_basis
                    .per_dollar
                    .pct50
                    .cents
                    .height,
            )?,
            sth_cointime_median_price_btc_weighted: MedianComponent::import(
                db,
                "sth_cointime_median_price_btc_weighted",
                version,
                mappings,
                &cointime
                    .urpd
                    .cohorts
                    .sth
                    .cost_basis
                    .per_coin
                    .pct50
                    .cents
                    .height,
            )?,
            sth_cointime_median_price_usd_weighted: MedianComponent::import(
                db,
                "sth_cointime_median_price_usd_weighted",
                version,
                mappings,
                &cointime
                    .urpd
                    .cohorts
                    .sth
                    .cost_basis
                    .per_dollar
                    .pct50
                    .cents
                    .height,
            )?,
            lth_cointime_median_price_btc_weighted: MedianComponent::import(
                db,
                "lth_cointime_median_price_btc_weighted",
                version,
                mappings,
                &cointime
                    .urpd
                    .cohorts
                    .lth
                    .cost_basis
                    .per_coin
                    .pct50
                    .cents
                    .height,
            )?,
            lth_cointime_median_price_usd_weighted: MedianComponent::import(
                db,
                "lth_cointime_median_price_usd_weighted",
                version,
                mappings,
                &cointime
                    .urpd
                    .cohorts
                    .lth
                    .cost_basis
                    .per_dollar
                    .pct50
                    .cents
                    .height,
            )?,
            sth_coinflow_median_price_btc_weighted: MedianComponent::import(
                db,
                "sth_coinflow_median_price_btc_weighted",
                version,
                mappings,
                &coinflow
                    .urpd
                    .cohorts
                    .sth
                    .cost_basis
                    .per_coin
                    .pct50
                    .cents
                    .height,
            )?,
            sth_coinflow_median_price_usd_weighted: MedianComponent::import(
                db,
                "sth_coinflow_median_price_usd_weighted",
                version,
                mappings,
                &coinflow
                    .urpd
                    .cohorts
                    .sth
                    .cost_basis
                    .per_dollar
                    .pct50
                    .cents
                    .height,
            )?,
            lth_coinflow_median_price_btc_weighted: MedianComponent::import(
                db,
                "lth_coinflow_median_price_btc_weighted",
                version,
                mappings,
                &coinflow
                    .urpd
                    .cohorts
                    .lth
                    .cost_basis
                    .per_coin
                    .pct50
                    .cents
                    .height,
            )?,
            lth_coinflow_median_price_usd_weighted: MedianComponent::import(
                db,
                "lth_coinflow_median_price_usd_weighted",
                version,
                mappings,
                &coinflow
                    .urpd
                    .cohorts
                    .lth
                    .cost_basis
                    .per_dollar
                    .pct50
                    .cents
                    .height,
            )?,
            sth_realized_price: import!("sth_realized_price", utxos.sth.realized.price),
            sth_capitalized_price: import!(
                "sth_capitalized_price",
                utxos.sth.realized.capitalized_price
            ),
            lth_realized_price: import!("lth_realized_price", utxos.lth.realized.price),
            lth_capitalized_price: import!(
                "lth_capitalized_price",
                utxos.lth.realized.capitalized_price
            ),
            over_6m_realized_price: import!("over_6m_realized_price", utxos.over_6m.realized.price),
            over_4m_realized_price: import!("over_4m_realized_price", utxos.over_4m.realized.price),
            under_4m_realized_price: import!(
                "under_4m_realized_price",
                utxos.under_4m.realized.price
            ),
            under_6m_realized_price: import!(
                "under_6m_realized_price",
                utxos.under_6m.realized.price
            ),
            under_4m_capitalized_price: import!(
                "under_4m_capitalized_price",
                utxos.under_4m.realized.capitalized_price
            ),
            under_6m_capitalized_price: import!(
                "under_6m_capitalized_price",
                utxos.under_6m.realized.capitalized_price
            ),
            vaulted_price: import!("vaulted_price", cointime.prices.vaulted),
            active_price: import!("active_price", cointime.prices.active),
            true_market_mean_price: import!(
                "true_market_mean_price",
                cointime.prices.true_market_mean
            ),
            cointime_price: import!("cointime_price", cointime.prices.cointime),
            awake_price: import!("awake_price", cointime.aggregate.all.awake.price),
            coinflow_price: import!("coinflow_price", coinflow.all.price),
        })
    }

    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        holders: &HoldersVecs,
        cointime: &CointimeVecs,
        coinflow: &CoinflowVecs,
        spot: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();
        let utxos = &holders.cohorts;

        [
            &mut self.median_price_btc_weighted,
            &mut self.median_price_usd_weighted,
            &mut self.sth_median_price_btc_weighted,
            &mut self.sth_median_price_usd_weighted,
            &mut self.lth_median_price_btc_weighted,
            &mut self.lth_median_price_usd_weighted,
            &mut self.cointime_median_price_btc_weighted,
            &mut self.cointime_median_price_usd_weighted,
            &mut self.coinflow_median_price_btc_weighted,
            &mut self.coinflow_median_price_usd_weighted,
            &mut self.sth_cointime_median_price_btc_weighted,
            &mut self.sth_cointime_median_price_usd_weighted,
            &mut self.lth_cointime_median_price_btc_weighted,
            &mut self.lth_cointime_median_price_usd_weighted,
            &mut self.sth_coinflow_median_price_btc_weighted,
            &mut self.sth_coinflow_median_price_usd_weighted,
            &mut self.lth_coinflow_median_price_btc_weighted,
            &mut self.lth_coinflow_median_price_usd_weighted,
        ]
        .into_par_iter()
        .try_for_each(|median| median.compute(&starting_lengths, spot, exit))?;

        let jobs = [
            (
                &mut self.realized_price,
                &utxos.all.realized.mvrv.ratio.height,
            ),
            (
                &mut self.capitalized_price,
                &utxos.all.realized.capitalized_price.relative.ratio.height,
            ),
            (
                &mut self.sth_realized_price,
                &utxos.sth.realized.mvrv.ratio.height,
            ),
            (
                &mut self.sth_capitalized_price,
                &utxos.sth.realized.capitalized_price.relative.ratio.height,
            ),
            (
                &mut self.lth_realized_price,
                &utxos.lth.realized.mvrv.ratio.height,
            ),
            (
                &mut self.lth_capitalized_price,
                &utxos.lth.realized.capitalized_price.relative.ratio.height,
            ),
            (
                &mut self.over_6m_realized_price,
                &utxos.over_6m.realized.mvrv.ratio.height,
            ),
            (
                &mut self.over_4m_realized_price,
                &utxos.over_4m.realized.mvrv.ratio.height,
            ),
            (
                &mut self.under_4m_realized_price,
                &utxos.under_4m.realized.mvrv.ratio.height,
            ),
            (
                &mut self.under_6m_realized_price,
                &utxos.under_6m.realized.mvrv.ratio.height,
            ),
            (
                &mut self.under_4m_capitalized_price,
                &utxos
                    .under_4m
                    .realized
                    .capitalized_price
                    .relative
                    .ratio
                    .height,
            ),
            (
                &mut self.under_6m_capitalized_price,
                &utxos
                    .under_6m
                    .realized
                    .capitalized_price
                    .relative
                    .ratio
                    .height,
            ),
            (
                &mut self.vaulted_price,
                &cointime.prices.vaulted.relative.ratio.height,
            ),
            (
                &mut self.active_price,
                &cointime.prices.active.relative.ratio.height,
            ),
            (
                &mut self.true_market_mean_price,
                &cointime.prices.true_market_mean.relative.ratio.height,
            ),
            (
                &mut self.cointime_price,
                &cointime.prices.cointime.relative.ratio.height,
            ),
            (
                &mut self.awake_price,
                &cointime.aggregate.all.awake.price.relative.ratio.height,
            ),
            (
                &mut self.coinflow_price,
                &coinflow.all.price.relative.ratio.height,
            ),
        ];
        let has_work = jobs
            .iter()
            .any(|(component, source)| component.needs_compute(starting_lengths.height, *source));
        let compute =
            |(component, source)| Component::compute(component, &starting_lengths, source, exit);

        if has_work {
            jobs.into_par_iter().try_for_each(compute)
        } else {
            jobs.into_iter().try_for_each(compute)
        }
    }
}
