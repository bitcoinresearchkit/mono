use bitview_plugin_indexer::Indexer;
use bitview_primitives::TypeIndex;
use brk_error::{Error, Result};
use brk_types::{OutputType, Sats};
use vecdb::ReadableVec;

use super::Batch;

/// Bulk txout reader; type indexes only `WITH_INDEXES`.
pub(super) struct TxOutReaders<'a, const WITH_INDEXES: bool> {
    indexer: &'a Indexer,
}

impl<'a, const WITH_INDEXES: bool> TxOutReaders<'a, WITH_INDEXES> {
    pub(super) fn new(indexer: &'a Indexer) -> Self {
        Self { indexer }
    }

    pub(super) fn collect_outputs(
        &self,
        batch: &Batch<'_>,
        values: &mut Vec<Sats>,
        output_types: &mut Vec<OutputType>,
        type_indexes: &mut Vec<TypeIndex>,
    ) -> Result<()> {
        let (first_txout_index, end) = (batch.outputs.start, batch.outputs.end);
        let output_count = batch.outputs.len();
        let outputs = &self.indexer.vecs().outputs;
        outputs
            .value
            .collect_range_into_at(first_txout_index, end, values);
        outputs
            .output_type
            .collect_range_into_at(first_txout_index, end, output_types);
        if WITH_INDEXES {
            outputs
                .type_index
                .collect_range_into_at(first_txout_index, end, type_indexes);
        }

        if values.len() != output_count
            || output_types.len() != output_count
            || (WITH_INDEXES && type_indexes.len() != output_count)
        {
            return Err(Error::NotFound(
                "incomplete distribution output columns".into(),
            ));
        }
        Ok(())
    }
}
