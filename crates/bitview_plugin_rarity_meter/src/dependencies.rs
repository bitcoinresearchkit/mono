use bitview_plugin_bedrock::Vecs as BedrockVecs;
use bitview_plugin_coinflow::Vecs as CoinflowVecs;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub bedrock: &'a BedrockVecs,
    pub holders: &'a HoldersVecs,
    pub cointime: &'a CointimeVecs,
    pub coinflow: &'a CoinflowVecs,
    pub price: &'a PriceVecs,
}
