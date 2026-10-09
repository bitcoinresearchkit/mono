use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_market::Vecs as MarketVecs;
use bitview_plugin_mining::Vecs as MiningVecs;
use bitview_plugin_utxos::Vecs as UtxosVecs;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub mining: &'a MiningVecs,
    pub utxos: &'a UtxosVecs,
    pub holders: &'a HoldersVecs,
    pub age: &'a AgeVecs,
    pub market: &'a MarketVecs,
}
