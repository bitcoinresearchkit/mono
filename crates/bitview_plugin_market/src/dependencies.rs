use brk_indexer::Indexer;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub price: &'a bitview_plugin_price::Vecs,
    pub indexes: &'a bitview_plugin_indexes::Vecs,
    pub blocks: &'a bitview_plugin_blocks::Vecs,
}
