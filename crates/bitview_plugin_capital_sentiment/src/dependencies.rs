use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_market::MovingAverageVecs;
use bitview_plugin_price::Vecs as PriceVecs;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub price: &'a PriceVecs,
    pub holders: &'a HoldersVecs,
    pub moving_average: &'a MovingAverageVecs,
}
