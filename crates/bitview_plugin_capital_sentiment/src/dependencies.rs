use bitview_plugin_distribution_aggregated::Vecs as AggregatedVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_market::MovingAverageVecs;
use bitview_plugin_price::Vecs as PriceVecs;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub price: &'a PriceVecs,
    pub distribution_aggregated: &'a AggregatedVecs,
    pub moving_average: &'a MovingAverageVecs,
}
