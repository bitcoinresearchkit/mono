use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_mining::Vecs as MiningVecs;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub mining: &'a MiningVecs,
    pub mappings: &'a MappingsVecs,
}
