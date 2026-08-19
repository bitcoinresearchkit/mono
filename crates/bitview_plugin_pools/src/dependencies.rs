use bitview_plugin_indexer::Indexer;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub indexes: &'a bitview_plugin_indexes::Vecs,
    pub price: &'a bitview_plugin_price::Vecs,
    pub mining: &'a bitview_plugin_mining::Vecs,
}
