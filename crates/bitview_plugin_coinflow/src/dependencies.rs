use bitview_plugin_indexer::Indexer;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub indexes: &'a bitview_plugin_indexes::Vecs,
    pub distribution: &'a bitview_plugin_distribution::Vecs,
}
