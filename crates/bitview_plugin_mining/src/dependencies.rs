use bitview_plugin_indexer::Indexer;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub indexes: &'a bitview_plugin_indexes::Vecs,
    pub blocks: &'a bitview_plugin_blocks::Vecs,
    pub transactions: &'a bitview_plugin_transactions::Vecs,
    pub price: &'a bitview_plugin_price::Vecs,
}
