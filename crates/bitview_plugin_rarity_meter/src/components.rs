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

/// The reference prices whose spot ratios the meter ranks, by the cohort they come from. Each
/// component's series follow its price's id: its bands `<price>_rarity_<pct>`
/// (`sth_median_realized_price_rarity_pct1`), their ratios `<price>_rarity_ratio_<pct>`; a median
/// also publishes its price and spot ratio (`sth_median_realized_price`, `..._ratio`).
#[derive(Traversable)]
pub struct Components<M: StorageMode = Rw> {
    /// All unspent outputs.
    pub all: CohortComponents<M>,
    /// Short-term holders: UTXOs younger than 150 days.
    pub sth: CohortComponents<M>,
    /// UTXOs less than 180 days old.
    pub under_6m: MeanComponents<M>,
    /// The awake supply.
    pub awake: AwakeComponents<M>,
    /// The mobile supply.
    pub mobile: MobileComponents<M>,
    /// Cointime vaulted price: realized price divided by one minus liveliness,
    /// where liveliness is cumulative coinblocks destroyed divided by
    /// cumulative coinblocks created.
    pub vaulted_price: Component<M>,
    /// Cointime True Market Mean: realized capitalization minus cumulative
    /// issuance-date subsidy value, divided by active supply; active supply is
    /// circulating supply multiplied by liveliness.
    pub true_market_mean: Component<M>,
}

/// A cohort's means and medians.
#[derive(Traversable)]
pub struct CohortComponents<M: StorageMode = Rw> {
    /// Realized price: the satoshi-weighted mean creation price.
    pub realized_price: Component<M>,
    /// Capitalized price: the mean creation price weighted by value invested at creation.
    pub capitalized_price: Component<M>,
    /// The median creation price, weighted by BTC supply.
    pub median_realized_price: MedianComponent<M>,
    /// The median creation price, weighted by creation-date USD value.
    pub median_capitalized_price: MedianComponent<M>,
}

/// A cohort's means.
#[derive(Traversable)]
pub struct MeanComponents<M: StorageMode = Rw> {
    /// Realized price: the satoshi-weighted mean creation price.
    pub realized_price: Component<M>,
    /// Capitalized price: the mean creation price weighted by value invested at creation.
    pub capitalized_price: Component<M>,
}

#[derive(Traversable)]
pub struct AwakeComponents<M: StorageMode = Rw> {
    /// Realized price: its wakefulness-weighted mean creation price.
    pub realized_price: Component<M>,
}

#[derive(Traversable)]
pub struct MobileComponents<M: StorageMode = Rw> {
    /// Realized price: realized capitalization weighted by each UTXO age range's estimated
    /// eventual spending probability, divided by supply weighted by the same probability.
    pub realized_price: Component<M>,
    /// The median creation price, weighted by BTC supply and each age range's mobility.
    pub median_realized_price: MedianComponent<M>,
    /// The median creation price, weighted by creation-date USD value and each age range's
    /// mobility.
    pub median_capitalized_price: MedianComponent<M>,
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
        // A mean's bands over its reference price; a median's price, spot ratio and bands over its
        // cost basis' median. Each named by its price's id.
        macro_rules! mean {
            ($price:literal, $reference:expr) => {
                Component::import(
                    db,
                    concat!($price, "_rarity"),
                    version,
                    mappings,
                    &$reference.cents.height,
                )?
            };
        }
        macro_rules! median {
            ($price:literal, $cost_basis:expr) => {
                MedianComponent::import(
                    db,
                    $price,
                    concat!($price, "_rarity"),
                    version,
                    mappings,
                    &$cost_basis.percentiles.median.cents.height,
                )?
            };
        }
        let cohorts = &holders.cohorts;
        let awake = &cointime.aggregate.cohorts.all.awake;
        let mobile = &coinflow.aggregate.cohorts.all.mobile;
        Ok(Self {
            all: CohortComponents {
                realized_price: mean!("realized_price", cohorts.all.cost_basis.per_coin.avg),
                capitalized_price: mean!(
                    "capitalized_price",
                    cohorts.all.cost_basis.per_dollar.avg
                ),
                median_realized_price: median!(
                    "median_realized_price",
                    cohorts.all.cost_basis.per_coin
                ),
                median_capitalized_price: median!(
                    "median_capitalized_price",
                    cohorts.all.cost_basis.per_dollar
                ),
            },
            sth: CohortComponents {
                realized_price: mean!("sth_realized_price", cohorts.sth.cost_basis.per_coin.avg),
                capitalized_price: mean!(
                    "sth_capitalized_price",
                    cohorts.sth.cost_basis.per_dollar.avg
                ),
                median_realized_price: median!(
                    "sth_median_realized_price",
                    cohorts.sth.cost_basis.per_coin
                ),
                median_capitalized_price: median!(
                    "sth_median_capitalized_price",
                    cohorts.sth.cost_basis.per_dollar
                ),
            },
            under_6m: MeanComponents {
                realized_price: mean!(
                    "utxos_under_6m_old_realized_price",
                    cohorts.under_6m.cost_basis.per_coin.avg
                ),
                capitalized_price: mean!(
                    "utxos_under_6m_old_capitalized_price",
                    cohorts.under_6m.cost_basis.per_dollar.avg
                ),
            },
            awake: AwakeComponents {
                realized_price: mean!("awake_realized_price", awake.cost_basis.per_coin.avg),
            },
            mobile: MobileComponents {
                realized_price: mean!("mobile_realized_price", mobile.cost_basis.per_coin.avg),
                median_realized_price: median!(
                    "mobile_median_realized_price",
                    mobile.cost_basis.per_coin
                ),
                median_capitalized_price: median!(
                    "mobile_median_capitalized_price",
                    mobile.cost_basis.per_dollar
                ),
            },
            vaulted_price: mean!("vaulted_price", cointime.prices.vaulted),
            true_market_mean: mean!("true_market_mean", cointime.prices.true_market_mean),
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
        let Self {
            all,
            sth,
            under_6m,
            awake,
            mobile,
            vaulted_price,
            true_market_mean,
        } = self;

        [
            &mut all.median_realized_price,
            &mut all.median_capitalized_price,
            &mut sth.median_realized_price,
            &mut sth.median_capitalized_price,
            &mut mobile.median_realized_price,
            &mut mobile.median_capitalized_price,
        ]
        .into_par_iter()
        .try_for_each(|median| median.compute(&starting_lengths, spot, exit))?;

        let cohorts = &holders.cohorts;
        let awake_cohort = &cointime.aggregate.cohorts.all.awake;
        let mobile_cohort = &coinflow.aggregate.cohorts.all.mobile;
        let jobs = [
            (
                &mut all.realized_price,
                &cohorts.all.cost_basis.per_coin.avg.relative.fixed.height,
            ),
            (
                &mut all.capitalized_price,
                &cohorts.all.cost_basis.per_dollar.avg.relative.fixed.height,
            ),
            (
                &mut sth.realized_price,
                &cohorts.sth.cost_basis.per_coin.avg.relative.fixed.height,
            ),
            (
                &mut sth.capitalized_price,
                &cohorts.sth.cost_basis.per_dollar.avg.relative.fixed.height,
            ),
            (
                &mut under_6m.realized_price,
                &cohorts
                    .under_6m
                    .cost_basis
                    .per_coin
                    .avg
                    .relative
                    .fixed
                    .height,
            ),
            (
                &mut under_6m.capitalized_price,
                &cohorts
                    .under_6m
                    .cost_basis
                    .per_dollar
                    .avg
                    .relative
                    .fixed
                    .height,
            ),
            (
                &mut awake.realized_price,
                &awake_cohort.cost_basis.per_coin.avg.relative.fixed.height,
            ),
            (
                &mut mobile.realized_price,
                &mobile_cohort.cost_basis.per_coin.avg.relative.fixed.height,
            ),
            (
                vaulted_price,
                &cointime.prices.vaulted.relative.fixed.height,
            ),
            (
                true_market_mean,
                &cointime.prices.true_market_mean.relative.fixed.height,
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

    pub(crate) fn local(&self) -> [&Component; 6] {
        [
            &self.sth.realized_price,
            &self.sth.capitalized_price,
            &self.under_6m.realized_price,
            &self.under_6m.capitalized_price,
            &self.sth.median_realized_price.component,
            &self.sth.median_capitalized_price.component,
        ]
    }

    pub(crate) fn cycle(&self) -> [&Component; 10] {
        [
            &self.all.realized_price,
            &self.all.capitalized_price,
            &self.all.median_realized_price.component,
            &self.all.median_capitalized_price.component,
            &self.mobile.median_realized_price.component,
            &self.mobile.median_capitalized_price.component,
            &self.vaulted_price,
            &self.true_market_mean,
            &self.awake.realized_price,
            &self.mobile.realized_price,
        ]
    }
}
