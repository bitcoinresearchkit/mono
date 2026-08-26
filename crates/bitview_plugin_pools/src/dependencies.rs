use bitview_plugin_indexer::Indexer;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub price: &'a bitview_plugin_price::Vecs,
    pub mining: &'a bitview_plugin_mining::Vecs,
}
