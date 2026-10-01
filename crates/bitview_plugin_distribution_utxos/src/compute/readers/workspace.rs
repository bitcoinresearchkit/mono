use bitview_plugin_distribution_common::readers::{TxInReaders, TxOutReaders};
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::HeightMap;
use brk_types::{Sats, TxInIndex, TxOutIndex};
use vecdb::PcoVec;

pub struct Workspace<'a> {
    pub outputs: TxOutReaders<'a, false>,
    pub inputs: TxInReaders<'a, false>,
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
        }
    }
}
