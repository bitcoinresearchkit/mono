use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Exit, Rw, StorageMode};

use super::{Component, component};
#[derive(Traversable)]
pub struct Components<M: StorageMode = Rw> {
    pub realized_price: Component<M>,
    pub capitalized_price: Component<M>,
    pub sth_realized_price: Component<M>,
    pub sth_capitalized_price: Component<M>,
    pub lth_realized_price: Component<M>,
    pub lth_capitalized_price: Component<M>,
    pub over_6m_realized_price: Component<M>,
    pub over_4m_realized_price: Component<M>,
    pub under_4m_realized_price: Component<M>,
    pub under_6m_realized_price: Component<M>,
    pub vaulted_price: Component<M>,
    pub active_price: Component<M>,
    pub true_market_mean_price: Component<M>,
    pub cointime_price: Component<M>,
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

        macro_rules! compute {
            ($component:ident, $source:expr) => {
                component::compute(
                    &mut self.$component,
                    &starting_lengths,
                    &$source.ratio.height,
                    exit,
                )?;
            };
        }

        compute!(realized_price, &realized_price.all);
        compute!(capitalized_price, &capitalized_price.all);
        compute!(sth_realized_price, &realized_price.term.short);
        compute!(sth_capitalized_price, &capitalized_price.sth);
        compute!(lth_realized_price, &realized_price.term.long);
        compute!(lth_capitalized_price, &capitalized_price.lth);
        compute!(over_6m_realized_price, &realized_price.age.over._6m);
        compute!(over_4m_realized_price, &realized_price.age.over._4m);
        compute!(under_4m_realized_price, &realized_price.age.under._4m);
        compute!(under_6m_realized_price, &realized_price.age.under._6m);
        compute!(vaulted_price, &cointime.prices.vaulted);
        compute!(active_price, &cointime.prices.active);
        compute!(true_market_mean_price, &cointime.prices.true_market_mean);
        compute!(cointime_price, &cointime.prices.cointime);
        compute!(coinflow_price, &coinflow.all.price);

        Ok(())
    }
}
