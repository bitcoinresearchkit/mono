use brk_indexer::Indexer;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub indexes: &'a bitview_plugin_indexes::Vecs,
    pub price: &'a bitview_plugin_price::Vecs,
    pub distribution: &'a bitview_plugin_distribution::Vecs,
    pub moving_average: &'a bitview_plugin_market::MovingAverageVecs,
}
