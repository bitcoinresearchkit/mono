use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::CachedSeries;
use brk_types::{Cents, Height};

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub mappings: &'a MappingsVecs,
    pub spot_cents: &'a CachedSeries<Height, Cents>,
}
