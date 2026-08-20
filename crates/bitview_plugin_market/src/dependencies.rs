use bitview_plugin_indexer::Indexer;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub price: &'a bitview_plugin_price::Vecs,
    pub mappings: &'a bitview_plugin_mappings::Vecs,
    pub blocks: &'a bitview_plugin_blocks::Vecs,
}
