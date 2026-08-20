#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]

use std::time::Instant;

use bitview_plugin_bedrock::Vecs as Bedrock;
use bitview_plugin_blocks::Vecs as Blocks;
use bitview_plugin_capital_sentiment::Vecs as CapitalSentiment;
use bitview_plugin_coinflow::Vecs as Coinflow;
use bitview_plugin_cointime::Vecs as Cointime;
use bitview_plugin_constants::Vecs as Constants;
use bitview_plugin_distribution::Vecs as Distribution;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_indicators::Vecs as Indicators;
use bitview_plugin_inputs::Vecs as Inputs;
use bitview_plugin_investing::Vecs as Investing;
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
use bitview_runtime::PluginSet;
use bitview_traversable::Traversable;
use tracing::info;
use vecdb::{Rw, StorageMode};

mod capabilities;
mod compute;
mod import;

#[derive(PluginSet, Traversable)]
pub struct DefaultPlugins<M: StorageMode = Rw> {
    #[traversable(flatten)]
    indexer: Box<Indexer<M>>,
    blocks: Box<Blocks<M>>,
    mining: Box<Mining<M>>,
    transactions: Box<Transactions<M>>,
    cointime: Box<Cointime<M>>,
    coinflow: Box<Coinflow<M>>,
    bedrock: Box<Bedrock<M>>,
    capital_sentiment: Box<CapitalSentiment<M>>,
    rarity_meter: Box<RarityMeter<M>>,
    constants: Box<Constants>,
    mappings: Box<Mappings<M>>,
    indicators: Box<Indicators<M>>,
    investing: Box<Investing>,
    market: Box<Market<M>>,
    pools: Box<Pools<M>>,
    price: Box<Price<M>>,
    #[traversable(flatten)]
    distribution: Box<Distribution<M>>,
    supply: Box<Supply<M>>,
    inputs: Box<Inputs<M>>,
    outputs: Box<Outputs<M>>,
    op_return: Box<OpReturn<M>>,
}

fn timed<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = f();
    info!("{label} in {:?}", start.elapsed());
    result
}
