use bitview_distribution::readers::Columns;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::HeightMap;
use bitview_primitives::{TxInIndex, TxOutIndex};
use brk_types::Sats;
use vecdb::PcoVec;

use crate::block::AddrCache;

/// Update-local source readers and scratch storage; address-state readers stay
/// chunk-local because the chunk writes that state.
pub struct Workspace<'a> {
    pub columns: Columns<'a, true>,
    pub addresses: AddrCache,
}

impl<'a> Workspace<'a> {
    pub fn new(
        indexer: &'a Indexer,
        values: &'a PcoVec<TxInIndex, Sats>,
        heights: &'a HeightMap<TxOutIndex>,
    ) -> Self {
        Self {
            columns: Columns::new(indexer, values, heights),
            addresses: AddrCache::default(),
        }
    }
}
