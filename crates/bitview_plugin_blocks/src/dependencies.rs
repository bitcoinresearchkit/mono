use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub mappings: &'a MappingsVecs,
}
