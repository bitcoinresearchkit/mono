use bitview_plugin_distribution_common::readers::Columns;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::HeightMap;
use brk_types::{Sats, TxInIndex, TxOutIndex};
use vecdb::PcoVec;

use crate::block::AddrCache;

use super::TxRanges;

/// Update-local source readers and scratch storage; address-state readers stay
/// chunk-local because the chunk writes that state.
pub struct Workspace<'a> {
    pub columns: Columns<'a, true>,
    pub output_txs: TxRanges,
    pub input_txs: TxRanges,
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
            output_txs: TxRanges::default(),
            input_txs: TxRanges::default(),
            addresses: AddrCache::default(),
        }
    }
}
