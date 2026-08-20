use bitview_plugin_indexer::Indexer;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub inputs: &'a bitview_plugin_inputs::Vecs,
    pub mappings: &'a bitview_plugin_mappings::Vecs,
    pub blocks: &'a bitview_plugin_blocks::Vecs,
    pub price: &'a bitview_plugin_price::Vecs,
}
