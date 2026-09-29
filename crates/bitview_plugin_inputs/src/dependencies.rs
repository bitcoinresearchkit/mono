use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as Mappings;

pub struct Dependencies<'a> {
    pub mappings: &'a Mappings,
    pub indexer: &'a Indexer,
    pub blocks: &'a BlocksVecs,
}
