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
    /// Rarity Meter component using the median creation price of UTXOs
    /// at least 150 days old, weighted by BTC supply.
    pub lth_cost_basis_per_coin_median: MedianComponent<M>,
    /// Rarity Meter component using the median creation price of UTXOs
    /// at least 150 days old, weighted by creation-date USD value.
    pub lth_cost_basis_per_dollar_median: MedianComponent<M>,
    /// Rarity Meter component using the awake (wakefulness-weighted) median creation price,
    /// weighted by BTC supply.
    pub awake_cost_basis_per_coin_median: MedianComponent<M>,
    /// Rarity Meter component using the awake (wakefulness-weighted) median creation price,
    /// weighted by creation-date USD value.
    pub awake_cost_basis_per_dollar_median: MedianComponent<M>,
    /// Rarity Meter component using the mobile (mobility-weighted) median creation price,
    /// weighted by BTC supply.
    pub mobile_cost_basis_per_coin_median: MedianComponent<M>,
    /// Rarity Meter component using the mobile (mobility-weighted) median creation price,
    /// weighted by creation-date USD value.
    pub mobile_cost_basis_per_dollar_median: MedianComponent<M>,
    /// STH awake (wakefulness-weighted) median creation price, per coin.
    pub sth_awake_cost_basis_per_coin_median: MedianComponent<M>,
    /// STH awake (wakefulness-weighted) median creation price, per dollar.
    pub sth_awake_cost_basis_per_dollar_median: MedianComponent<M>,
    /// LTH awake (wakefulness-weighted) median creation price, per coin.
    pub lth_awake_cost_basis_per_coin_median: MedianComponent<M>,
    /// LTH awake (wakefulness-weighted) median creation price, per dollar.
    pub lth_awake_cost_basis_per_dollar_median: MedianComponent<M>,
    /// STH mobile (mobility-weighted) median creation price, per coin.
    pub sth_mobile_cost_basis_per_coin_median: MedianComponent<M>,
    /// STH mobile (mobility-weighted) median creation price, per dollar.
    pub sth_mobile_cost_basis_per_dollar_median: MedianComponent<M>,
    /// LTH mobile (mobility-weighted) median creation price, per coin.
    pub lth_mobile_cost_basis_per_coin_median: MedianComponent<M>,
    /// LTH mobile (mobility-weighted) median creation price, per dollar.
    pub lth_mobile_cost_basis_per_dollar_median: MedianComponent<M>,
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
    pub utxos_over_6m_old_realized_price: Component<M>,
    /// Rarity Meter component using the satoshi-weighted mean creation price of
    /// UTXOs at least 120 days old as its reference.
    pub utxos_over_4m_old_realized_price: Component<M>,
    /// Rarity Meter component using the satoshi-weighted mean creation price of
    /// UTXOs less than 120 days old as its reference.
    pub utxos_under_4m_old_realized_price: Component<M>,
    /// Rarity Meter component using the satoshi-weighted mean creation price of
    /// UTXOs less than 180 days old as its reference.
    pub utxos_under_6m_old_realized_price: Component<M>,
    /// Rarity Meter component using the value-weighted mean creation price of
    /// UTXOs less than 120 days old as its reference.
    pub utxos_under_4m_old_capitalized_price: Component<M>,
    /// Rarity Meter component using the value-weighted mean creation price of
    /// UTXOs less than 180 days old as its reference.
    pub utxos_under_6m_old_capitalized_price: Component<M>,
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
    pub true_market_mean: Component<M>,
    /// Rarity Meter component using cointime price as its reference: the
    /// cumulative sum of spot price multiplied by coinblocks destroyed, divided
    /// by cumulative coinblocks stored.
    pub cointime_price: Component<M>,
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

        macro_rules! import {
            ($name:expr, $source:expr) => {
                Component::import(db, $name, version, mappings, &$source.cents.height)?
            };
        }

        Ok(Components {
            realized_price: import!("realized_price", utxos.all.cost_basis.per_coin.avg),
            capitalized_price: import!("capitalized_price", utxos.all.cost_basis.per_dollar.avg),
            cost_basis_per_coin_median: MedianComponent::import(
                db,
                "cost_basis_per_coin_median",
                version,
                mappings,
                &utxos
                    .all
                    .cost_basis
                    .per_coin
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            cost_basis_per_dollar_median: MedianComponent::import(
                db,
                "cost_basis_per_dollar_median",
                version,
                mappings,
                &utxos
                    .all
                    .cost_basis
                    .per_dollar
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            sth_cost_basis_per_coin_median: MedianComponent::import(
                db,
                "sth_cost_basis_per_coin_median",
                version,
                mappings,
                &utxos
                    .sth
                    .cost_basis
                    .per_coin
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            sth_cost_basis_per_dollar_median: MedianComponent::import(
                db,
                "sth_cost_basis_per_dollar_median",
                version,
                mappings,
                &utxos
                    .sth
                    .cost_basis
                    .per_dollar
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            lth_cost_basis_per_coin_median: MedianComponent::import(
                db,
                "lth_cost_basis_per_coin_median",
                version,
                mappings,
                &utxos
                    .lth
                    .cost_basis
                    .per_coin
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            lth_cost_basis_per_dollar_median: MedianComponent::import(
                db,
                "lth_cost_basis_per_dollar_median",
                version,
                mappings,
                &utxos
                    .lth
                    .cost_basis
                    .per_dollar
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            awake_cost_basis_per_coin_median: MedianComponent::import(
                db,
                "awake_cost_basis_per_coin_median",
                version,
                mappings,
                &cointime
                    .aggregate
                    .cohorts
                    .all
                    .awake
                    .cost_basis
                    .per_coin
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            awake_cost_basis_per_dollar_median: MedianComponent::import(
                db,
                "awake_cost_basis_per_dollar_median",
                version,
                mappings,
                &cointime
                    .aggregate
                    .cohorts
                    .all
                    .awake
                    .cost_basis
                    .per_dollar
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            mobile_cost_basis_per_coin_median: MedianComponent::import(
                db,
                "mobile_cost_basis_per_coin_median",
                version,
                mappings,
                &coinflow
                    .aggregate
                    .cohorts
                    .all
                    .mobile
                    .cost_basis
                    .per_coin
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            mobile_cost_basis_per_dollar_median: MedianComponent::import(
                db,
                "mobile_cost_basis_per_dollar_median",
                version,
                mappings,
                &coinflow
                    .aggregate
                    .cohorts
                    .all
                    .mobile
                    .cost_basis
                    .per_dollar
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            sth_awake_cost_basis_per_coin_median: MedianComponent::import(
                db,
                "sth_awake_cost_basis_per_coin_median",
                version,
                mappings,
                &cointime
                    .aggregate
                    .cohorts
                    .sth
                    .awake
                    .cost_basis
                    .per_coin
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            sth_awake_cost_basis_per_dollar_median: MedianComponent::import(
                db,
                "sth_awake_cost_basis_per_dollar_median",
                version,
                mappings,
                &cointime
                    .aggregate
                    .cohorts
                    .sth
                    .awake
                    .cost_basis
                    .per_dollar
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            lth_awake_cost_basis_per_coin_median: MedianComponent::import(
                db,
                "lth_awake_cost_basis_per_coin_median",
                version,
                mappings,
                &cointime
                    .aggregate
                    .cohorts
                    .lth
                    .awake
                    .cost_basis
                    .per_coin
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            lth_awake_cost_basis_per_dollar_median: MedianComponent::import(
                db,
                "lth_awake_cost_basis_per_dollar_median",
                version,
                mappings,
                &cointime
                    .aggregate
                    .cohorts
                    .lth
                    .awake
                    .cost_basis
                    .per_dollar
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            sth_mobile_cost_basis_per_coin_median: MedianComponent::import(
                db,
                "sth_mobile_cost_basis_per_coin_median",
                version,
                mappings,
                &coinflow
                    .aggregate
                    .cohorts
                    .sth
                    .mobile
                    .cost_basis
                    .per_coin
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            sth_mobile_cost_basis_per_dollar_median: MedianComponent::import(
                db,
                "sth_mobile_cost_basis_per_dollar_median",
                version,
                mappings,
                &coinflow
                    .aggregate
                    .cohorts
                    .sth
                    .mobile
                    .cost_basis
                    .per_dollar
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            lth_mobile_cost_basis_per_coin_median: MedianComponent::import(
                db,
                "lth_mobile_cost_basis_per_coin_median",
                version,
                mappings,
                &coinflow
                    .aggregate
                    .cohorts
                    .lth
                    .mobile
                    .cost_basis
                    .per_coin
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            lth_mobile_cost_basis_per_dollar_median: MedianComponent::import(
                db,
                "lth_mobile_cost_basis_per_dollar_median",
                version,
                mappings,
                &coinflow
                    .aggregate
                    .cohorts
                    .lth
                    .mobile
                    .cost_basis
                    .per_dollar
                    .percentiles
                    .median
                    .cents
                    .height,
            )?,
            sth_realized_price: import!("sth_realized_price", utxos.sth.cost_basis.per_coin.avg),
            sth_capitalized_price: import!(
                "sth_capitalized_price",
                utxos.sth.cost_basis.per_dollar.avg
            ),
            lth_realized_price: import!("lth_realized_price", utxos.lth.cost_basis.per_coin.avg),
            lth_capitalized_price: import!(
                "lth_capitalized_price",
                utxos.lth.cost_basis.per_dollar.avg
            ),
            utxos_over_6m_old_realized_price: import!(
                "utxos_over_6m_old_realized_price",
                utxos.over_6m.cost_basis.per_coin.avg
            ),
            utxos_over_4m_old_realized_price: import!(
                "utxos_over_4m_old_realized_price",
                utxos.over_4m.cost_basis.per_coin.avg
            ),
            utxos_under_4m_old_realized_price: import!(
                "utxos_under_4m_old_realized_price",
                utxos.under_4m.cost_basis.per_coin.avg
            ),
            utxos_under_6m_old_realized_price: import!(
                "utxos_under_6m_old_realized_price",
                utxos.under_6m.cost_basis.per_coin.avg
            ),
            utxos_under_4m_old_capitalized_price: import!(
                "utxos_under_4m_old_capitalized_price",
                utxos.under_4m.cost_basis.per_dollar.avg
            ),
            utxos_under_6m_old_capitalized_price: import!(
                "utxos_under_6m_old_capitalized_price",
                utxos.under_6m.cost_basis.per_dollar.avg
            ),
            vaulted_price: import!("vaulted_price", cointime.prices.vaulted),
            active_price: import!("active_price", cointime.prices.active),
            true_market_mean: import!("true_market_mean", cointime.prices.true_market_mean),
            cointime_price: import!("cointime_price", cointime.prices.cointime),
            awake_realized_price: import!(
                "awake_realized_price",
                cointime.aggregate.cohorts.all.awake.cost_basis.per_coin.avg
            ),
            mobile_realized_price: import!(
                "mobile_realized_price",
                coinflow
                    .aggregate
                    .cohorts
                    .all
                    .mobile
                    .cost_basis
                    .per_coin
                    .avg
            ),
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
            &mut self.cost_basis_per_coin_median,
            &mut self.cost_basis_per_dollar_median,
            &mut self.sth_cost_basis_per_coin_median,
            &mut self.sth_cost_basis_per_dollar_median,
            &mut self.lth_cost_basis_per_coin_median,
            &mut self.lth_cost_basis_per_dollar_median,
            &mut self.awake_cost_basis_per_coin_median,
            &mut self.awake_cost_basis_per_dollar_median,
            &mut self.mobile_cost_basis_per_coin_median,
            &mut self.mobile_cost_basis_per_dollar_median,
            &mut self.sth_awake_cost_basis_per_coin_median,
            &mut self.sth_awake_cost_basis_per_dollar_median,
            &mut self.lth_awake_cost_basis_per_coin_median,
            &mut self.lth_awake_cost_basis_per_dollar_median,
            &mut self.sth_mobile_cost_basis_per_coin_median,
            &mut self.sth_mobile_cost_basis_per_dollar_median,
            &mut self.lth_mobile_cost_basis_per_coin_median,
            &mut self.lth_mobile_cost_basis_per_dollar_median,
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
                &mut self.lth_realized_price,
                &utxos.lth.cost_basis.per_coin.avg.relative.ratio.height,
            ),
            (
                &mut self.lth_capitalized_price,
                &utxos.lth.cost_basis.per_dollar.avg.relative.ratio.height,
            ),
            (
                &mut self.utxos_over_6m_old_realized_price,
                &utxos.over_6m.cost_basis.per_coin.avg.relative.ratio.height,
            ),
            (
                &mut self.utxos_over_4m_old_realized_price,
                &utxos.over_4m.cost_basis.per_coin.avg.relative.ratio.height,
            ),
            (
                &mut self.utxos_under_4m_old_realized_price,
                &utxos.under_4m.cost_basis.per_coin.avg.relative.ratio.height,
            ),
            (
                &mut self.utxos_under_6m_old_realized_price,
                &utxos.under_6m.cost_basis.per_coin.avg.relative.ratio.height,
            ),
            (
                &mut self.utxos_under_4m_old_capitalized_price,
                &utxos
                    .under_4m
                    .cost_basis
                    .per_dollar
                    .avg
                    .relative
                    .ratio
                    .height,
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
                &mut self.active_price,
                &cointime.prices.active.relative.ratio.height,
            ),
            (
                &mut self.true_market_mean,
                &cointime.prices.true_market_mean.relative.ratio.height,
            ),
            (
                &mut self.cointime_price,
                &cointime.prices.cointime.relative.ratio.height,
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
                &coinflow
                    .aggregate
                    .cohorts
                    .all
                    .mobile
                    .cost_basis
                    .per_coin
                    .avg
                    .relative
                    .ratio
                    .height,
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
