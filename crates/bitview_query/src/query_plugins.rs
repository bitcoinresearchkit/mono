#[cfg(feature = "urpd")]
use bitview_plugin_bedrock::{HasBedrock, Vecs as Bedrock};
#[cfg(feature = "chain")]
use bitview_plugin_blocks::{HasBlocks, Vecs as Blocks};
#[cfg(feature = "urpd")]
use bitview_plugin_coinflow::{HasCoinflow, Vecs as Coinflow};
#[cfg(feature = "urpd")]
use bitview_plugin_cointime::{HasCointime, Vecs as Cointime};
#[cfg(any(feature = "chain", feature = "urpd"))]
use bitview_plugin_distribution::{HasDistribution, Vecs as Distribution};
use bitview_plugin_indexer::{HasIndexer, Indexer};
#[cfg(any(feature = "chain", feature = "series"))]
use bitview_plugin_indexes::{HasIndexes, Vecs as Indexes};
#[cfg(feature = "chain")]
use bitview_plugin_inputs::{HasInputs, Vecs as Inputs};
#[cfg(feature = "chain")]
use bitview_plugin_mining::{HasMining, Vecs as Mining};
#[cfg(feature = "chain")]
use bitview_plugin_outputs::{HasOutputs, Vecs as Outputs};
#[cfg(feature = "chain")]
use bitview_plugin_pools::{HasPools, Vecs as Pools};
#[cfg(any(feature = "chain", feature = "urpd"))]
use bitview_plugin_price::{HasPrice, Vecs as Price};
#[cfg(feature = "chain")]
use bitview_plugin_transactions::{HasTransactions, Vecs as Transactions};
use vecdb::Ro;

use crate::QueryPluginSet;

pub struct QueryPlugins<'a> {
    pub indexer: &'a Indexer<Ro>,
    #[cfg(any(feature = "chain", feature = "urpd"))]
    pub distribution: &'a Distribution<Ro>,
    #[cfg(any(feature = "chain", feature = "series"))]
    pub indexes: &'a Indexes<Ro>,
    #[cfg(feature = "chain")]
    pub blocks: &'a Blocks<Ro>,
    #[cfg(feature = "chain")]
    pub inputs: &'a Inputs<Ro>,
    #[cfg(feature = "chain")]
    pub mining: &'a Mining<Ro>,
    #[cfg(feature = "chain")]
    pub outputs: &'a Outputs<Ro>,
    #[cfg(feature = "chain")]
    pub pools: &'a Pools<Ro>,
    #[cfg(any(feature = "chain", feature = "urpd"))]
    pub price: &'a Price<Ro>,
    #[cfg(feature = "chain")]
    pub transactions: &'a Transactions<Ro>,
    #[cfg(feature = "urpd")]
    pub bedrock: &'a Bedrock<Ro>,
    #[cfg(feature = "urpd")]
    pub coinflow: &'a Coinflow<Ro>,
    #[cfg(feature = "urpd")]
    pub cointime: &'a Cointime<Ro>,
}

impl<'a> QueryPlugins<'a> {
    pub fn new<P>(plugins: &'a P) -> Self
    where
        P: QueryPluginSet,
    {
        let plugins = plugins.query_capabilities();

        Self {
            indexer: plugins.indexer(),
            #[cfg(any(feature = "chain", feature = "urpd"))]
            distribution: plugins.distribution(),
            #[cfg(any(feature = "chain", feature = "series"))]
            indexes: plugins.indexes(),
            #[cfg(feature = "chain")]
            blocks: plugins.blocks(),
            #[cfg(feature = "chain")]
            inputs: plugins.inputs(),
            #[cfg(feature = "chain")]
            mining: plugins.mining(),
            #[cfg(feature = "chain")]
            outputs: plugins.outputs(),
            #[cfg(feature = "chain")]
            pools: plugins.pools(),
            #[cfg(any(feature = "chain", feature = "urpd"))]
            price: plugins.price(),
            #[cfg(feature = "chain")]
            transactions: plugins.transactions(),
            #[cfg(feature = "urpd")]
            bedrock: plugins.bedrock(),
            #[cfg(feature = "urpd")]
            coinflow: plugins.coinflow(),
            #[cfg(feature = "urpd")]
            cointime: plugins.cointime(),
        }
    }
}
