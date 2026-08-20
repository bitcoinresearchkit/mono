use bitview_plugin_indexer::Indexer;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub mappings: &'a bitview_plugin_mappings::Vecs,
    pub distribution: &'a bitview_plugin_distribution::Vecs,
}
