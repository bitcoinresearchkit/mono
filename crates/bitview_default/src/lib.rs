#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]

use bitview_plugin_bedrock::Vecs as Bedrock;
use bitview_plugin_blocks::Vecs as Blocks;
use bitview_plugin_capital_sentiment::Vecs as CapitalSentiment;
use bitview_plugin_coinflow::Vecs as Coinflow;
use bitview_plugin_cointime::Vecs as Cointime;
use bitview_plugin_constants::Vecs as Constants;
use bitview_plugin_distribution_addresses::Vecs as DistributionAddresses;
use bitview_plugin_distribution_age::Vecs as DistributionAge;
use bitview_plugin_distribution_aggregated::Vecs as DistributionAggregated;
use bitview_plugin_distribution_utxos::Vecs as DistributionUtxos;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_indicators::Vecs as Indicators;
use bitview_plugin_inputs::Vecs as Inputs;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_plugin_market::Vecs as Market;
use bitview_plugin_mining::Vecs as Mining;
use bitview_plugin_op_return::Vecs as OpReturn;
use bitview_plugin_outputs::Vecs as Outputs;
use bitview_plugin_pools::Vecs as Pools;
use bitview_plugin_price::Vecs as Price;
use bitview_plugin_rarity_meter::Vecs as RarityMeter;
use bitview_plugin_supply::Vecs as Supply;
use bitview_plugin_transactions::Vecs as Transactions;
use bitview_plugin_utxo_history::Vecs as UtxoHistory;
use bitview_runtime::PluginSet;
use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

mod compute;
mod import;
mod timing;

#[derive(PluginSet, Traversable)]
pub struct DefaultPlugins<M: StorageMode = Rw> {
    #[plugin_set(has = bitview_plugin_indexer::HasIndexer<M>)]
    #[traversable(flatten)]
    indexer: Box<Indexer<M>>,
    #[plugin_set(has = bitview_plugin_blocks::HasBlocks<M>)]
    blocks: Box<Blocks<M>>,
    #[plugin_set(has = bitview_plugin_mining::HasMining<M>)]
    mining: Box<Mining<M>>,
    #[plugin_set(has = bitview_plugin_transactions::HasTransactions<M>)]
    transactions: Box<Transactions<M>>,
    #[plugin_set(has = bitview_plugin_cointime::HasCointime<M>)]
    cointime: Box<Cointime<M>>,
    #[plugin_set(has = bitview_plugin_coinflow::HasCoinflow<M>)]
    coinflow: Box<Coinflow<M>>,
    bedrock: Box<Bedrock<M>>,
    capital_sentiment: Box<CapitalSentiment<M>>,
    rarity_meter: Box<RarityMeter<M>>,
    constants: Box<Constants>,
    #[plugin_set(has = bitview_plugin_mappings::HasMappings<M>)]
    mappings: Box<Mappings<M>>,
    indicators: Box<Indicators<M>>,
    market: Box<Market<M>>,
    #[plugin_set(has = bitview_plugin_pools::HasPools<M>)]
    pools: Box<Pools<M>>,
    #[plugin_set(has = bitview_plugin_price::HasPrice<M>)]
    price: Box<Price<M>>,
    #[plugin_set(has = bitview_plugin_distribution_age::HasDistributionAge<M>)]
    #[traversable(flatten)]
    distribution_age: Box<DistributionAge<M>>,
    #[plugin_set(has = bitview_plugin_distribution_aggregated::HasDistributionAggregated<M>)]
    distribution_aggregated: Box<DistributionAggregated<M>>,
    #[traversable(flatten)]
    distribution_utxos: Box<DistributionUtxos<M>>,
    #[plugin_set(has = bitview_plugin_distribution_addresses::HasDistributionAddresses<M>)]
    #[traversable(flatten)]
    distribution_addresses: Box<DistributionAddresses<M>>,
    supply: Box<Supply<M>>,
    #[plugin_set(has = bitview_plugin_inputs::HasInputs<M>)]
    inputs: Box<Inputs<M>>,
    #[plugin_set(has = bitview_plugin_outputs::HasOutputs<M>)]
    outputs: Box<Outputs<M>>,
    #[plugin_set(has = bitview_plugin_utxo_history::HasUtxoHistory<M>)]
    utxo_history: Box<UtxoHistory<M>>,
    op_return: Box<OpReturn<M>>,
}
