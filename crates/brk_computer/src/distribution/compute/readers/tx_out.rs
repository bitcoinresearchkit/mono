use brk_indexer::Indexer;
use brk_types::{OutputType, Sats, TypeIndex};
use vecdb::ReadableVec;

use super::TxOutData;

/// Bulk txout reader with reusable buffers.
pub struct TxOutReaders<'a> {
    indexer: &'a Indexer,
    values_buf: Vec<Sats>,
    output_types_buf: Vec<OutputType>,
    type_indexes_buf: Vec<TypeIndex>,
    txout_data_buf: Vec<TxOutData>,
}

impl<'a> TxOutReaders<'a> {
    pub(crate) fn new(indexer: &'a Indexer) -> Self {
        Self {
            indexer,
            values_buf: Vec::new(),
            output_types_buf: Vec::new(),
            type_indexes_buf: Vec::new(),
            txout_data_buf: Vec::new(),
        }
    }

    pub(crate) fn collect_block_outputs(
        &mut self,
        first_txout_index: usize,
        output_count: usize,
    ) -> &[TxOutData] {
        let end = first_txout_index + output_count;
        self.indexer.vecs().outputs.value.collect_range_into_at(
            first_txout_index,
            end,
            &mut self.values_buf,
        );
        self.indexer
            .vecs()
            .outputs
            .output_type
            .collect_range_into_at(first_txout_index, end, &mut self.output_types_buf);
        self.indexer
            .vecs()
            .outputs
            .type_index
            .collect_range_into_at(first_txout_index, end, &mut self.type_indexes_buf);

        self.txout_data_buf.clear();
        self.txout_data_buf.extend(
            self.values_buf
                .iter()
                .zip(&self.output_types_buf)
                .zip(&self.type_indexes_buf)
                .map(|((&value, &output_type), &type_index)| TxOutData {
                    value,
                    output_type,
                    type_index,
                }),
        );
        &self.txout_data_buf
    }
}
