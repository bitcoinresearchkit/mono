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
    /// All-chain realized price: the satoshi-weighted mean creation price of all
    /// unspent outputs.
    pub realized_price: Component<M>,
    /// All-chain capitalized price: the mean creation price weighted by value
    /// invested at creation.
    pub capitalized_price: Component<M>,
    /// All-chain median creation price, weighted by BTC supply.
    pub cost_basis_per_coin_median: MedianComponent<M>,
    /// All-chain median creation price, weighted by creation-date USD value.
    pub cost_basis_per_dollar_median: MedianComponent<M>,
    /// Median creation price of UTXOs younger than 150 days, weighted by BTC
    /// supply.
    pub sth_cost_basis_per_coin_median: MedianComponent<M>,
    /// Median creation price of UTXOs younger than 150 days, weighted by
    /// creation-date USD value.
    pub sth_cost_basis_per_dollar_median: MedianComponent<M>,
    /// Median creation price weighted by BTC supply and each age range's mobility.
    pub mobile_cost_basis_per_coin_median: MedianComponent<M>,
    /// Median creation price weighted by creation-date USD value and each age
    /// range's mobility.
    pub mobile_cost_basis_per_dollar_median: MedianComponent<M>,
    /// Satoshi-weighted mean creation price of UTXOs younger than 150 days.
    pub sth_realized_price: Component<M>,
    /// Value-weighted mean creation price of UTXOs younger than 150 days.
    pub sth_capitalized_price: Component<M>,
    /// Satoshi-weighted mean creation price of UTXOs less than 180 days old.
    pub utxos_under_6m_old_realized_price: Component<M>,
    /// Value-weighted mean creation price of UTXOs less than 180 days old.
    pub utxos_under_6m_old_capitalized_price: Component<M>,
    /// Cointime vaulted price: realized price divided by one minus liveliness,
    /// where liveliness is cumulative coinblocks destroyed divided by
    /// cumulative coinblocks created.
    pub vaulted_price: Component<M>,
    /// Cointime True Market Mean: realized capitalization minus cumulative
    /// issuance-date subsidy value, divided by active supply; active supply is
    /// circulating supply multiplied by liveliness.
    pub true_market_mean: Component<M>,
    /// The awake supply's realized price: its wakefulness-weighted mean creation
    /// price.
    pub awake_realized_price: Component<M>,
    /// The mobile supply's realized price: realized capitalization weighted by
    /// each UTXO age range's estimated eventual spending probability, divided
    /// by supply weighted by the same probability.
    pub mobile_realized_price: Component<M>,
}

/// Generates the import and compute of every component from one table: each mean component's
/// reference price (its cents and its spot ratio), and each median component's cost basis (whose
/// median it prices). `$holders`, `$cointime` and `$coinflow` name the plugins the paths start at.
macro_rules! components {
    (
        ($holders:ident, $cointime:ident, $coinflow:ident)
        means { $($mean:ident: $reference:expr,)* }
        medians { $($median:ident: $cost_basis:expr,)* }
    ) => {
        impl Components {
            pub(crate) fn import(
                db: &Database,
                version: Version,
                mappings: &MappingsVecs,
                $holders: &HoldersVecs,
                $cointime: &CointimeVecs,
                $coinflow: &CoinflowVecs,
            ) -> Result<Self> {
                Ok(Components {
                    $($mean: Component::import(
                        db,
                        stringify!($mean),
                        version,
                        mappings,
                        &$reference.cents.height,
                    )?,)*
                    $($median: MedianComponent::import(
                        db,
                        stringify!($median),
                        version,
                        mappings,
                        &$cost_basis.percentiles.median.cents.height,
                    )?,)*
                })
            }

            pub(crate) fn compute(
                &mut self,
                indexer: &Indexer,
                $holders: &HoldersVecs,
                $cointime: &CointimeVecs,
                $coinflow: &CoinflowVecs,
                spot: &impl ReadableVec<Height, Cents>,
                exit: &Exit,
            ) -> Result<()> {
                let starting_lengths = indexer.compute_start();

                [$(&mut self.$median,)*]
                    .into_par_iter()
                    .try_for_each(|median| median.compute(&starting_lengths, spot, exit))?;

                let jobs = [$((&mut self.$mean, &$reference.relative.ratio.height),)*];
                let has_work = jobs.iter().any(|(component, source)| {
                    component.needs_compute(starting_lengths.height, *source)
                });
                let compute = |(component, source)| {
                    Component::compute(component, &starting_lengths, source, exit)
                };

                if has_work {
                    jobs.into_par_iter().try_for_each(compute)
                } else {
                    jobs.into_iter().try_for_each(compute)
                }
            }
        }
    };
}

components! {
    (holders, cointime, coinflow)
    means {
        realized_price: holders.cohorts.all.cost_basis.per_coin.avg,
        capitalized_price: holders.cohorts.all.cost_basis.per_dollar.avg,
        sth_realized_price: holders.cohorts.sth.cost_basis.per_coin.avg,
        sth_capitalized_price: holders.cohorts.sth.cost_basis.per_dollar.avg,
        utxos_under_6m_old_realized_price: holders.cohorts.under_6m.cost_basis.per_coin.avg,
        utxos_under_6m_old_capitalized_price: holders.cohorts.under_6m.cost_basis.per_dollar.avg,
        vaulted_price: cointime.prices.vaulted,
        true_market_mean: cointime.prices.true_market_mean,
        awake_realized_price: cointime.aggregate.cohorts.all.awake.cost_basis.per_coin.avg,
        mobile_realized_price: coinflow.aggregate.cohorts.all.mobile.cost_basis.per_coin.avg,
    }
    medians {
        cost_basis_per_coin_median: holders.cohorts.all.cost_basis.per_coin,
        cost_basis_per_dollar_median: holders.cohorts.all.cost_basis.per_dollar,
        sth_cost_basis_per_coin_median: holders.cohorts.sth.cost_basis.per_coin,
        sth_cost_basis_per_dollar_median: holders.cohorts.sth.cost_basis.per_dollar,
        mobile_cost_basis_per_coin_median: coinflow.aggregate.cohorts.all.mobile.cost_basis.per_coin,
        mobile_cost_basis_per_dollar_median: coinflow.aggregate.cohorts.all.mobile.cost_basis.per_dollar,
    }
}

impl Components {
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
