#[cfg(feature = "chain")]
use bitview_plugin_blocks::{HasBlocks, Vecs as Blocks};
#[cfg(feature = "urpd")]
use bitview_plugin_coinflow::{HasCoinflow, Vecs as Coinflow};
#[cfg(feature = "urpd")]
use bitview_plugin_cointime::{HasCointime, Vecs as Cointime};
#[cfg(feature = "chain")]
use bitview_plugin_distribution_addresses::{
    HasDistributionAddresses, Vecs as DistributionAddresses,
};
#[cfg(feature = "urpd")]
use bitview_plugin_distribution_age::{HasDistributionAge, Vecs as DistributionAge};
use bitview_plugin_indexer::{HasIndexer, Indexer};
#[cfg(feature = "chain")]
use bitview_plugin_inputs::{HasInputs, Vecs as Inputs};
#[cfg(any(feature = "series", feature = "price"))]
use bitview_plugin_mappings::{HasMappings, Vecs as Mappings};
#[cfg(feature = "chain")]
use bitview_plugin_mining::{HasMining, Vecs as Mining};
#[cfg(feature = "chain")]
use bitview_plugin_outputs::{HasOutputs, Vecs as Outputs};
#[cfg(feature = "chain")]
use bitview_plugin_pools::{HasPools, Vecs as Pools};
#[cfg(feature = "price")]
use bitview_plugin_price::{HasPrice, Vecs as Price};
#[cfg(feature = "chain")]
use bitview_plugin_transactions::{HasTransactions, Vecs as Transactions};
#[cfg(any(feature = "chain", feature = "urpd"))]
use bitview_plugin_utxo_history::{HasUtxoHistory, Vecs as UtxoHistory};
use vecdb::Ro;

use crate::QueryPluginSet;

pub struct QueryPlugins<'a> {
    #[cfg(feature = "chain")]
    pub(crate) distribution_addresses: &'a DistributionAddresses<Ro>,
    pub(crate) indexer: &'a Indexer<Ro>,
    #[cfg(feature = "urpd")]
    pub(crate) distribution_age: &'a DistributionAge<Ro>,
    #[cfg(any(feature = "series", feature = "price"))]
    pub(crate) mappings: &'a Mappings<Ro>,
    #[cfg(feature = "chain")]
    pub(crate) blocks: &'a Blocks<Ro>,
    #[cfg(feature = "chain")]
    pub(crate) inputs: &'a Inputs<Ro>,
    #[cfg(feature = "chain")]
    pub(crate) mining: &'a Mining<Ro>,
    #[cfg(feature = "chain")]
    pub(crate) outputs: &'a Outputs<Ro>,
    #[cfg(any(feature = "chain", feature = "urpd"))]
    pub(crate) utxo_history: &'a UtxoHistory<Ro>,
    #[cfg(feature = "chain")]
    pub(crate) pools: &'a Pools<Ro>,
    #[cfg(feature = "price")]
    pub(crate) price: &'a Price<Ro>,
    #[cfg(feature = "chain")]
    pub(crate) transactions: &'a Transactions<Ro>,
    #[cfg(feature = "urpd")]
    pub(crate) coinflow: &'a Coinflow<Ro>,
    #[cfg(feature = "urpd")]
    pub(crate) cointime: &'a Cointime<Ro>,
}

impl<'a> QueryPlugins<'a> {
    pub(crate) fn new<P>(plugins: &'a P) -> Self
    where
        P: QueryPluginSet,
    {
        let plugins = plugins.query_capabilities();

        Self {
            indexer: plugins.indexer(),
            #[cfg(feature = "chain")]
            distribution_addresses: plugins.distribution_addresses(),
            #[cfg(feature = "urpd")]
            distribution_age: plugins.distribution_age(),
            #[cfg(any(feature = "series", feature = "price"))]
            mappings: plugins.mappings(),
            #[cfg(feature = "chain")]
            blocks: plugins.blocks(),
            #[cfg(feature = "chain")]
            inputs: plugins.inputs(),
            #[cfg(feature = "chain")]
            mining: plugins.mining(),
            #[cfg(feature = "chain")]
            outputs: plugins.outputs(),
            #[cfg(any(feature = "chain", feature = "urpd"))]
            utxo_history: plugins.utxo_history(),
            #[cfg(feature = "chain")]
            pools: plugins.pools(),
            #[cfg(feature = "price")]
            price: plugins.price(),
            #[cfg(feature = "chain")]
            transactions: plugins.transactions(),
            #[cfg(feature = "urpd")]
            coinflow: plugins.coinflow(),
            #[cfg(feature = "urpd")]
            cointime: plugins.cointime(),
        }
    }
}
