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
    pub cost_basis_per_coin_median: MedianComponent<M>,
    /// Rarity Meter component using the median creation price,
    /// weighted by creation-date USD value.
    pub cost_basis_per_dollar_median: MedianComponent<M>,
    /// Rarity Meter component using the median creation price of UTXOs
    /// younger than 150 days, weighted by BTC supply.
    pub sth_cost_basis_per_coin_median: MedianComponent<M>,
    /// Rarity Meter component using the median creation price of UTXOs
    /// younger than 150 days, weighted by creation-date USD value.
    pub sth_cost_basis_per_dollar_median: MedianComponent<M>,
    /// Rarity Meter component using the mobile (mobility-weighted) median creation price,
    /// weighted by BTC supply.
    pub mobile_cost_basis_per_coin_median: MedianComponent<M>,
    /// Rarity Meter component using the mobile (mobility-weighted) median creation price,
    /// weighted by creation-date USD value.
    pub mobile_cost_basis_per_dollar_median: MedianComponent<M>,
    /// Rarity Meter component using the satoshi-weighted mean creation price of
    /// UTXOs younger than 150 days as its reference.
    pub sth_realized_price: Component<M>,
    /// Rarity Meter component using the value-weighted mean creation price of
    /// UTXOs younger than 150 days as its reference.
    pub sth_capitalized_price: Component<M>,
    /// Rarity Meter component using the satoshi-weighted mean creation price of
    /// UTXOs less than 180 days old as its reference.
    pub utxos_under_6m_old_realized_price: Component<M>,
    /// Rarity Meter component using the value-weighted mean creation price of
    /// UTXOs less than 180 days old as its reference.
    pub utxos_under_6m_old_capitalized_price: Component<M>,
    /// Rarity Meter component using cointime vaulted price as its reference:
    /// realized price divided by one minus liveliness, where liveliness is
    /// cumulative coinblocks destroyed divided by cumulative coinblocks
    /// created.
    pub vaulted_price: Component<M>,
    /// Rarity Meter component using cointime true market mean price as its
    /// reference: realized capitalization minus cumulative issuance-date
    /// subsidy value, divided by active supply; active supply is circulating
    /// supply multiplied by liveliness.
    pub true_market_mean: Component<M>,
    /// Rarity Meter component using the awake supply's realized price, its
    /// wakefulness-weighted mean creation price.
    pub awake_realized_price: Component<M>,
    /// Rarity Meter component using the mobile supply's realized price: realized
    /// capitalization weighted by each UTXO age range's estimated eventual
    /// spending probability, divided by supply weighted by the same
    /// probability.
    pub mobile_realized_price: Component<M>,
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
        let mobile = &coinflow.aggregate.cohorts.all.mobile;

        macro_rules! import {
            ($name:expr, $source:expr) => {
                Component::import(db, $name, version, mappings, &$source.cents.height)?
            };
        }
        macro_rules! median {
            ($name:expr, $cost_basis:expr) => {
                MedianComponent::import(
                    db,
                    $name,
                    version,
                    mappings,
                    &$cost_basis.percentiles.median.cents.height,
                )?
            };
        }

        Ok(Components {
            realized_price: import!("realized_price", utxos.all.cost_basis.per_coin.avg),
            capitalized_price: import!("capitalized_price", utxos.all.cost_basis.per_dollar.avg),
            cost_basis_per_coin_median: median!(
                "cost_basis_per_coin_median",
                utxos.all.cost_basis.per_coin
            ),
            cost_basis_per_dollar_median: median!(
                "cost_basis_per_dollar_median",
                utxos.all.cost_basis.per_dollar
            ),
            sth_cost_basis_per_coin_median: median!(
                "sth_cost_basis_per_coin_median",
                utxos.sth.cost_basis.per_coin
            ),
            sth_cost_basis_per_dollar_median: median!(
                "sth_cost_basis_per_dollar_median",
                utxos.sth.cost_basis.per_dollar
            ),
            mobile_cost_basis_per_coin_median: median!(
                "mobile_cost_basis_per_coin_median",
                mobile.cost_basis.per_coin
            ),
            mobile_cost_basis_per_dollar_median: median!(
                "mobile_cost_basis_per_dollar_median",
                mobile.cost_basis.per_dollar
            ),
            sth_realized_price: import!("sth_realized_price", utxos.sth.cost_basis.per_coin.avg),
            sth_capitalized_price: import!(
                "sth_capitalized_price",
                utxos.sth.cost_basis.per_dollar.avg
            ),
            utxos_under_6m_old_realized_price: import!(
                "utxos_under_6m_old_realized_price",
                utxos.under_6m.cost_basis.per_coin.avg
            ),
            utxos_under_6m_old_capitalized_price: import!(
                "utxos_under_6m_old_capitalized_price",
                utxos.under_6m.cost_basis.per_dollar.avg
            ),
            vaulted_price: import!("vaulted_price", cointime.prices.vaulted),
            true_market_mean: import!("true_market_mean", cointime.prices.true_market_mean),
            awake_realized_price: import!(
                "awake_realized_price",
                cointime.aggregate.cohorts.all.awake.cost_basis.per_coin.avg
            ),
            mobile_realized_price: import!("mobile_realized_price", mobile.cost_basis.per_coin.avg),
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
        let starting_lengths = indexer.compute_start();
        let utxos = &holders.cohorts;
        let mobile = &coinflow.aggregate.cohorts.all.mobile;

        [
            &mut self.cost_basis_per_coin_median,
            &mut self.cost_basis_per_dollar_median,
            &mut self.sth_cost_basis_per_coin_median,
            &mut self.sth_cost_basis_per_dollar_median,
            &mut self.mobile_cost_basis_per_coin_median,
            &mut self.mobile_cost_basis_per_dollar_median,
        ]
        .into_par_iter()
        .try_for_each(|median| median.compute(&starting_lengths, spot, exit))?;

        let jobs = [
            (
                &mut self.realized_price,
                &utxos.all.cost_basis.per_coin.avg.relative.ratio.height,
            ),
            (
                &mut self.capitalized_price,
                &utxos.all.cost_basis.per_dollar.avg.relative.ratio.height,
            ),
            (
                &mut self.sth_realized_price,
                &utxos.sth.cost_basis.per_coin.avg.relative.ratio.height,
            ),
            (
                &mut self.sth_capitalized_price,
                &utxos.sth.cost_basis.per_dollar.avg.relative.ratio.height,
            ),
            (
                &mut self.utxos_under_6m_old_realized_price,
                &utxos.under_6m.cost_basis.per_coin.avg.relative.ratio.height,
            ),
            (
                &mut self.utxos_under_6m_old_capitalized_price,
                &utxos
                    .under_6m
                    .cost_basis
                    .per_dollar
                    .avg
                    .relative
                    .ratio
                    .height,
            ),
            (
                &mut self.vaulted_price,
                &cointime.prices.vaulted.relative.ratio.height,
            ),
            (
                &mut self.true_market_mean,
                &cointime.prices.true_market_mean.relative.ratio.height,
            ),
            (
                &mut self.awake_realized_price,
                &cointime
                    .aggregate
                    .cohorts
                    .all
                    .awake
                    .cost_basis
                    .per_coin
                    .avg
                    .relative
                    .ratio
                    .height,
            ),
            (
                &mut self.mobile_realized_price,
                &mobile.cost_basis.per_coin.avg.relative.ratio.height,
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

    /// Young-coin references: the short-term holders' and under-six-month cohorts' means and
    /// the short-term holders' medians.
    pub(crate) fn local(&self) -> [&Component; 6] {
        [
            &self.sth_realized_price,
            &self.sth_capitalized_price,
            &self.utxos_under_6m_old_realized_price,
            &self.utxos_under_6m_old_capitalized_price,
            &self.sth_cost_basis_per_coin_median.component,
            &self.sth_cost_basis_per_dollar_median.component,
        ]
    }

    /// Long-cycle references: all-chain means and medians, the mobile supply's medians and the
    /// Cointime and Coinflow prices.
    pub(crate) fn cycle(&self) -> [&Component; 10] {
        [
            &self.realized_price,
            &self.capitalized_price,
            &self.cost_basis_per_coin_median.component,
            &self.cost_basis_per_dollar_median.component,
            &self.mobile_cost_basis_per_coin_median.component,
            &self.mobile_cost_basis_per_dollar_median.component,
            &self.vaulted_price,
            &self.true_market_mean,
            &self.awake_realized_price,
            &self.mobile_realized_price,
        ]
    }
}
