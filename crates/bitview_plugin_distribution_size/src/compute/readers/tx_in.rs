use bitview_plugin_mappings::TxHeightMap;
use brk_types::{Height, OutPoint, OutputType, Sats, TxInIndex, TypeIndex};
use vecdb::{Cursor, PcoVec, ReadableVec};

#[cfg(test)]
mod tests;

/// Bulk txin reader with reusable buffers.
pub struct TxInReaders<'a> {
    input_values: Cursor<'a, TxInIndex, Sats, PcoVec<TxInIndex, Sats>>,
    outpoints: Cursor<'a, TxInIndex, OutPoint, PcoVec<TxInIndex, OutPoint>>,
    output_types: Cursor<'a, TxInIndex, OutputType, PcoVec<TxInIndex, OutputType>>,
    type_indexes: Cursor<'a, TxInIndex, TypeIndex, PcoVec<TxInIndex, TypeIndex>>,
    tx_index_to_height: &'a TxHeightMap,
    outpoints_buf: Vec<OutPoint>,
    values_buf: Vec<Sats>,
    prev_heights_buf: Vec<Height>,
    output_types_buf: Vec<OutputType>,
    type_indexes_buf: Vec<TypeIndex>,
}

impl<'a> TxInReaders<'a> {
    pub fn new(
        input_values: &'a PcoVec<TxInIndex, Sats>,
        outpoints: &'a PcoVec<TxInIndex, OutPoint>,
        output_types: &'a PcoVec<TxInIndex, OutputType>,
        type_indexes: &'a PcoVec<TxInIndex, TypeIndex>,
        tx_index_to_height: &'a TxHeightMap,
    ) -> Self {
        Self {
            input_values: input_values.cursor(),
            outpoints: outpoints.cursor(),
            output_types: output_types.cursor(),
            type_indexes: type_indexes.cursor(),
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
        self.outpoints
            .collect_range_into_at(first_txin_index, end, &mut self.outpoints_buf);
        self.output_types
            .collect_range_into_at(first_txin_index, end, &mut self.output_types_buf);
        self.type_indexes
            .collect_range_into_at(first_txin_index, end, &mut self.type_indexes_buf);

        self.prev_heights_buf.clear();
        self.prev_heights_buf
            .extend(self.outpoints_buf.iter().map(|outpoint| {
                if outpoint.is_coinbase() {
                    current_height
                } else {
                    self.tx_index_to_height
                        .get_shared(outpoint.tx_index())
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
