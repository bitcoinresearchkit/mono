use bitview_plugin_mappings::HeightMap;
use brk_error::{Error, Result};
use brk_types::{Height, OutputType, Sats, TxInIndex, TxOutIndex, TypeIndex};
use vecdb::{Cursor, PcoVec, ReadableVec};

#[cfg(test)]
mod tests;

/// Bulk txin reader with reusable buffers.
pub struct TxInReaders<'a, const WITH_INDEXES: bool = true> {
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
    pub fn new(
        input_values: &'a PcoVec<TxInIndex, Sats>,
        references: &'a PcoVec<TxInIndex, TxOutIndex>,
        output_types: &'a PcoVec<TxInIndex, OutputType>,
        type_indexes: &'a PcoVec<TxInIndex, TypeIndex>,
        output_heights: &'a HeightMap<TxOutIndex>,
    ) -> Self {
        Self {
            input_values: input_values.cursor(),
            references: references.cursor(),
            output_types: output_types.cursor(),
            type_indexes: type_indexes.cursor(),
            output_heights,
            values_buf: Vec::new(),
            prev_heights_buf: Vec::new(),
            output_types_buf: Vec::new(),
            type_indexes_buf: Vec::new(),
        }
    }

    pub fn collect_inputs(
        &mut self,
        first_txin_index: usize,
        input_count: usize,
        current_height: Height,
    ) -> Result<(&[Sats], &[Height], &[OutputType], &[TypeIndex])> {
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
