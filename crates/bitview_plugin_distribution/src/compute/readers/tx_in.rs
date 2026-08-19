use bitview_plugin_indexer::Indexer;
use brk_types::{Height, OutPoint, OutputType, RangeMap, Sats, TxInIndex, TxIndex, TypeIndex};
use vecdb::{PcoVec, ReadableVec};

/// Bulk txin reader with reusable buffers.
pub struct TxInReaders<'a> {
    indexer: &'a Indexer,
    input_values: &'a PcoVec<TxInIndex, Sats>,
    tx_index_to_height: &'a mut RangeMap<TxIndex, Height>,
    outpoints_buf: Vec<OutPoint>,
    values_buf: Vec<Sats>,
    prev_heights_buf: Vec<Height>,
    output_types_buf: Vec<OutputType>,
    type_indexes_buf: Vec<TypeIndex>,
}

impl<'a> TxInReaders<'a> {
    pub fn new(
        indexer: &'a Indexer,
        input_values: &'a PcoVec<TxInIndex, Sats>,
        tx_index_to_height: &'a mut RangeMap<TxIndex, Height>,
    ) -> Self {
        Self {
            indexer,
            input_values,
            tx_index_to_height,
            outpoints_buf: Vec::new(),
            values_buf: Vec::new(),
            prev_heights_buf: Vec::new(),
            output_types_buf: Vec::new(),
            type_indexes_buf: Vec::new(),
        }
    }

    pub fn collect_block_inputs(
        &mut self,
        first_txin_index: usize,
        input_count: usize,
        current_height: Height,
    ) -> (&[Sats], &[Height], &[OutputType], &[TypeIndex]) {
        let end = first_txin_index + input_count;
        self.input_values
            .collect_range_into_at(first_txin_index, end, &mut self.values_buf);
        self.indexer.vecs().inputs.outpoint.collect_range_into_at(
            first_txin_index,
            end,
            &mut self.outpoints_buf,
        );
        self.indexer
            .vecs()
            .inputs
            .output_type
            .collect_range_into_at(first_txin_index, end, &mut self.output_types_buf);
        self.indexer.vecs().inputs.type_index.collect_range_into_at(
            first_txin_index,
            end,
            &mut self.type_indexes_buf,
        );

        self.prev_heights_buf.clear();
        self.prev_heights_buf
            .extend(self.outpoints_buf.iter().map(|outpoint| {
                if outpoint.is_coinbase() {
                    current_height
                } else {
                    self.tx_index_to_height
                        .get(outpoint.tx_index())
                        .unwrap_or(current_height)
                }
            }));

        (
            &self.values_buf,
            &self.prev_heights_buf,
            &self.output_types_buf,
            &self.type_indexes_buf,
        )
    }
}
