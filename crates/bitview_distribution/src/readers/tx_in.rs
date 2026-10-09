use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::HeightMap;
use bitview_primitives::{TxInIndex, TxOutIndex, TypeIndex};
use brk_error::{Error, Result};
use brk_types::{Height, OutputType, Sats};
use vecdb::{Cursor, PcoVec, ReadableVec};

use super::Batch;

/// Sequential txin reader; type indexes only `WITH_INDEXES`.
pub(super) struct TxInReaders<'a, const WITH_INDEXES: bool> {
    input_values: Cursor<'a, TxInIndex, Sats, PcoVec<TxInIndex, Sats>>,
    references: Cursor<'a, TxInIndex, TxOutIndex, PcoVec<TxInIndex, TxOutIndex>>,
    output_types: Cursor<'a, TxInIndex, OutputType, PcoVec<TxInIndex, OutputType>>,
    type_indexes: Cursor<'a, TxInIndex, TypeIndex, PcoVec<TxInIndex, TypeIndex>>,
    output_heights: &'a HeightMap<TxOutIndex>,
}

impl<'a, const WITH_INDEXES: bool> TxInReaders<'a, WITH_INDEXES> {
    pub(super) fn new(
        indexer: &'a Indexer,
        input_values: &'a PcoVec<TxInIndex, Sats>,
        output_heights: &'a HeightMap<TxOutIndex>,
    ) -> Self {
        let inputs = &indexer.vecs().inputs;
        Self {
            input_values: input_values.cursor(),
            references: inputs.txout_index.cursor(),
            output_types: inputs.output_type.cursor(),
            type_indexes: inputs.type_index.cursor(),
            output_heights,
        }
    }

    pub(super) fn collect_inputs(
        &mut self,
        batch: &Batch<'_>,
        values: &mut Vec<Sats>,
        prev_heights: &mut Vec<Height>,
        output_types: &mut Vec<OutputType>,
        type_indexes: &mut Vec<TypeIndex>,
    ) -> Result<()> {
        let (first_txin_index, end) = (batch.inputs.start, batch.inputs.end);
        let input_count = batch.inputs.len();
        let current_height = Height::from(batch.blocks.start);
        self.input_values
            .collect_range_into_at(first_txin_index, end, values);
        self.output_types
            .collect_range_into_at(first_txin_index, end, output_types);
        if WITH_INDEXES {
            self.type_indexes
                .collect_range_into_at(first_txin_index, end, type_indexes);
        }

        prev_heights.clear();
        prev_heights.reserve(input_count);
        self.references
            .try_for_each_range_at(first_txin_index, end, |reference| {
                prev_heights.push(if reference.is_coinbase() {
                    current_height
                } else {
                    self.output_heights
                        .get_shared(reference)
                        .ok_or_else(|| Error::NotFound("spent output creation height".into()))?
                });
                Ok::<_, Error>(())
            })?;
        if values.len() != input_count
            || prev_heights.len() != input_count
            || output_types.len() != input_count
            || (WITH_INDEXES && type_indexes.len() != input_count)
        {
            return Err(Error::NotFound(
                "incomplete distribution input columns".into(),
            ));
        }
        Ok(())
    }
}
