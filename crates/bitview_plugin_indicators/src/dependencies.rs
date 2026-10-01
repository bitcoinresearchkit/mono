use bitview_plugin_distribution_age::Vecs as AgeVecs;
use bitview_plugin_distribution_aggregated::Vecs as AggregatedVecs;
use bitview_plugin_distribution_utxos::Vecs as UtxosVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_market::Vecs as MarketVecs;
use bitview_plugin_mining::Vecs as MiningVecs;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub mining: &'a MiningVecs,
    pub utxos: &'a UtxosVecs,
    pub distribution_aggregated: &'a AggregatedVecs,
    pub distribution_age: &'a AgeVecs,
    pub market: &'a MarketVecs,
}
