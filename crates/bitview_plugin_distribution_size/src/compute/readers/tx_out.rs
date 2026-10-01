use bitview_plugin_indexer::Indexer;
use brk_error::{Error, Result};
use brk_types::{OutputType, Sats, TypeIndex};
use vecdb::ReadableVec;

/// Bulk txout reader with reusable buffers.
pub struct TxOutReaders<'a> {
    indexer: &'a Indexer,
    values_buf: Vec<Sats>,
    output_types_buf: Vec<OutputType>,
    type_indexes_buf: Vec<TypeIndex>,
}

impl<'a> TxOutReaders<'a> {
    pub fn new(indexer: &'a Indexer) -> Self {
        Self {
            indexer,
            values_buf: Vec::new(),
            output_types_buf: Vec::new(),
            type_indexes_buf: Vec::new(),
        }
    }

    pub fn collect_outputs(
        &mut self,
        first_txout_index: usize,
        output_count: usize,
    ) -> Result<(&[Sats], &[OutputType], &[TypeIndex])> {
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

        if self.values_buf.len() != output_count
            || self.output_types_buf.len() != output_count
            || self.type_indexes_buf.len() != output_count
        {
            return Err(Error::NotFound("incomplete Size output columns".into()));
        }
        Ok((
            &self.values_buf,
            &self.output_types_buf,
            &self.type_indexes_buf,
        ))
    }
}
