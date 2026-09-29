use bitview_plugin_distribution_age::Vecs as AgeVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_market::MovingAverageVecs;
use bitview_plugin_price::Vecs as PriceVecs;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub price: &'a PriceVecs,
    pub distribution_age: &'a AgeVecs,
    pub moving_average: &'a MovingAverageVecs,
}
