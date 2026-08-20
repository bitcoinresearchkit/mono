use bitview_plugin_indexer::Indexer;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub mappings: &'a bitview_plugin_mappings::Vecs,
    pub inputs: &'a bitview_plugin_inputs::Vecs,
    pub outputs: &'a bitview_plugin_outputs::Vecs,
    pub transactions: &'a bitview_plugin_transactions::Vecs,
    pub price: &'a bitview_plugin_price::Vecs,
}
