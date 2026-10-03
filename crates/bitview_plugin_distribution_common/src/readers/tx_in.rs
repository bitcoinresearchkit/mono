use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::HeightMap;
use bitview_primitives::{TxInIndex, TxOutIndex, TypeIndex};
use brk_error::{Error, Result};
use brk_types::{Height, OutputType, Sats};
use vecdb::{Cursor, PcoVec, ReadableVec};

pub(super) type InputColumns<'a> = (&'a [Sats], &'a [Height], &'a [OutputType], &'a [TypeIndex]);

/// Bulk txin reader with reusable buffers.
pub(super) struct TxInReaders<'a, const WITH_INDEXES: bool> {
    input_values: Cursor<'a, TxInIndex, Sats, PcoVec<TxInIndex, Sats>>,
    references: Cursor<'a, TxInIndex, TxOutIndex, PcoVec<TxInIndex, TxOutIndex>>,
    output_types: Cursor<'a, TxInIndex, OutputType, PcoVec<TxInIndex, OutputType>>,
    type_indexes: Cursor<'a, TxInIndex, TypeIndex, PcoVec<TxInIndex, TypeIndex>>,
    output_heights: &'a HeightMap<TxOutIndex>,
    values_buf: Vec<Sats>,
    prev_heights_buf: Vec<Height>,
    output_types_buf: Vec<OutputType>,
    type_indexes_buf: Vec<TypeIndex>,
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
            values_buf: Vec::new(),
            prev_heights_buf: Vec::new(),
            output_types_buf: Vec::new(),
            type_indexes_buf: Vec::new(),
        }
    }

    pub(super) fn collect_inputs(
        &mut self,
        first_txin_index: usize,
        input_count: usize,
        current_height: Height,
    ) -> Result<InputColumns<'_>> {
        let end = first_txin_index + input_count;
        self.input_values
            .collect_range_into_at(first_txin_index, end, &mut self.values_buf);
        self.output_types
            .collect_range_into_at(first_txin_index, end, &mut self.output_types_buf);
        if WITH_INDEXES {
            self.type_indexes.collect_range_into_at(
                first_txin_index,
                end,
                &mut self.type_indexes_buf,
            );
        }

        self.prev_heights_buf.clear();
        self.prev_heights_buf.reserve(input_count);
        self.references
            .try_for_each_range_at(first_txin_index, end, |reference| {
                self.prev_heights_buf.push(if reference.is_coinbase() {
                    current_height
                } else {
                    self.output_heights
                        .get_shared(reference)
                        .ok_or_else(|| Error::NotFound("spent output creation height".into()))?
                });
                Ok::<_, Error>(())
            })?;
        if self.values_buf.len() != input_count
            || self.prev_heights_buf.len() != input_count
            || self.output_types_buf.len() != input_count
            || (WITH_INDEXES && self.type_indexes_buf.len() != input_count)
        {
            return Err(Error::NotFound(
                "incomplete distribution input columns".into(),
            ));
        }

        Ok((
            &self.values_buf,
            &self.prev_heights_buf,
            &self.output_types_buf,
            &self.type_indexes_buf,
        ))
    }
}
