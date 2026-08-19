use bitview_plugin_indexer::Indexer;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub distribution: &'a bitview_plugin_distribution::Vecs,
    pub cointime: &'a bitview_plugin_cointime::Vecs,
    pub coinflow: &'a bitview_plugin_coinflow::Vecs,
    pub price: &'a bitview_plugin_price::Vecs,
}
