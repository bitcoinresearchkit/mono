use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::HeightMap;
use brk_types::{Sats, TxInIndex, TxOutIndex};
use vecdb::PcoVec;

use crate::block::AddrCache;

use super::{TxInReaders, TxOutReaders, TxRanges};

/// Update-local source readers and scratch storage; address-state readers stay
/// chunk-local because the chunk writes that state.
pub struct Workspace<'a> {
    pub outputs: TxOutReaders<'a>,
    pub inputs: TxInReaders<'a>,
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
        let inputs = &indexer.vecs().inputs;
        Self {
            outputs: TxOutReaders::new(indexer),
            inputs: TxInReaders::new(
                values,
                &inputs.txout_index,
                &inputs.output_type,
                &inputs.type_index,
                heights,
            ),
            output_txs: TxRanges::default(),
            input_txs: TxRanges::default(),
            addresses: AddrCache::default(),
        }
    }
}
