use bitview_plugin_distribution_age::Vecs as AgeVecs;
use bitview_plugin_distribution_size::Vecs as SizeVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_market::Vecs as MarketVecs;
use bitview_plugin_mining::Vecs as MiningVecs;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub mining: &'a MiningVecs,
    pub size: &'a SizeVecs,
    pub distribution_age: &'a AgeVecs,
    pub market: &'a MarketVecs,
}
