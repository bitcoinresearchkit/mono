use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_urpd::ReplayInputs;

#[derive(Clone, Copy)]
pub struct Dependencies<'a> {
    pub urpd: ReplayInputs<'a>,
    pub indexer: &'a Indexer,
    pub mappings: &'a MappingsVecs,
    pub age: &'a AgeVecs,
}
