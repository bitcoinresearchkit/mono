use bitview_plugin_indexer::Indexer;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub mappings: &'a bitview_plugin_mappings::Vecs,
    pub price: &'a bitview_plugin_price::Vecs,
    pub mining: &'a bitview_plugin_mining::Vecs,
}
