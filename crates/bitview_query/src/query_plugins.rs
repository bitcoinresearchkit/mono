#[cfg(feature = "bedrock")]
use bitview_plugin_bedrock::{HasBedrock, Vecs as Bedrock};
#[cfg(feature = "blocks")]
use bitview_plugin_blocks::{HasBlocks, Vecs as Blocks};
#[cfg(feature = "coinflow")]
use bitview_plugin_coinflow::{HasCoinflow, Vecs as Coinflow};
#[cfg(feature = "cointime")]
use bitview_plugin_cointime::{HasCointime, Vecs as Cointime};
#[cfg(feature = "distribution")]
use bitview_plugin_distribution::{HasDistribution, Vecs as Distribution};
use bitview_plugin_indexer::{HasIndexer, Indexer};
#[cfg(feature = "inputs")]
use bitview_plugin_inputs::{HasInputs, Vecs as Inputs};
#[cfg(feature = "mappings")]
use bitview_plugin_mappings::{HasMappings, Vecs as Mappings};
#[cfg(feature = "mining")]
use bitview_plugin_mining::{HasMining, Vecs as Mining};
#[cfg(feature = "outputs")]
use bitview_plugin_outputs::{HasOutputs, Vecs as Outputs};
#[cfg(feature = "pools")]
use bitview_plugin_pools::{HasPools, Vecs as Pools};
#[cfg(feature = "price")]
use bitview_plugin_price::{HasPrice, Vecs as Price};
#[cfg(feature = "transactions")]
use bitview_plugin_transactions::{HasTransactions, Vecs as Transactions};
use vecdb::Ro;

use crate::QueryPluginSet;

pub struct QueryPlugins<'a> {
    pub indexer: &'a Indexer<Ro>,
    #[cfg(feature = "distribution")]
    pub distribution: &'a Distribution<Ro>,
    #[cfg(feature = "mappings")]
    pub mappings: &'a Mappings<Ro>,
    #[cfg(feature = "blocks")]
    pub blocks: &'a Blocks<Ro>,
    #[cfg(feature = "inputs")]
    pub inputs: &'a Inputs<Ro>,
    #[cfg(feature = "mining")]
    pub mining: &'a Mining<Ro>,
    #[cfg(feature = "outputs")]
    pub outputs: &'a Outputs<Ro>,
    #[cfg(feature = "pools")]
    pub pools: &'a Pools<Ro>,
    #[cfg(feature = "price")]
    pub price: &'a Price<Ro>,
    #[cfg(feature = "transactions")]
    pub transactions: &'a Transactions<Ro>,
    #[cfg(feature = "bedrock")]
    pub bedrock: &'a Bedrock<Ro>,
    #[cfg(feature = "coinflow")]
    pub coinflow: &'a Coinflow<Ro>,
    #[cfg(feature = "cointime")]
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
            #[cfg(feature = "distribution")]
            distribution: plugins.distribution(),
            #[cfg(feature = "mappings")]
            mappings: plugins.mappings(),
            #[cfg(feature = "blocks")]
            blocks: plugins.blocks(),
            #[cfg(feature = "inputs")]
            inputs: plugins.inputs(),
            #[cfg(feature = "mining")]
            mining: plugins.mining(),
            #[cfg(feature = "outputs")]
            outputs: plugins.outputs(),
            #[cfg(feature = "pools")]
            pools: plugins.pools(),
            #[cfg(feature = "price")]
            price: plugins.price(),
            #[cfg(feature = "transactions")]
            transactions: plugins.transactions(),
            #[cfg(feature = "bedrock")]
            bedrock: plugins.bedrock(),
            #[cfg(feature = "coinflow")]
            coinflow: plugins.coinflow(),
            #[cfg(feature = "cointime")]
            cointime: plugins.cointime(),
        }
    }
}
