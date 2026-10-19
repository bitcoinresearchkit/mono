use bitview_distribution::readers::ColumnReaders;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::HeightMap;
use bitview_primitives::TxOutIndex;

use crate::block::AddrCache;

/// Update-local source readers and scratch storage; address-state readers stay
/// chunk-local because the chunk writes that state.
pub struct Workspace<'a> {
    pub columns: ColumnReaders<'a, true>,
    pub addresses: AddrCache,
}

impl<'a> Workspace<'a> {
    pub fn new(indexer: &'a Indexer, heights: &'a HeightMap<TxOutIndex>) -> Self {
        Self {
            columns: ColumnReaders::new(indexer, heights),
            addresses: AddrCache::default(),
        }
    }
}
