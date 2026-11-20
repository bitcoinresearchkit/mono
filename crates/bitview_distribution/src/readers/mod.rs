mod tx_in;
use std::{
    mem,
    ops::{ControlFlow, Range},
};

use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::HeightMap;
use bitview_primitives::{Boolean, TxInIndex, TxOutIndex, TypeIndex};
use brk_error::{Error, Result};
use brk_types::{Height, OutputType, Sats};
use rayon::join;
use vecdb::{AnyVec, ReadableVec, VecIndex};

use tx_in::TxInReaders;

/// Blocks read together, bounding source buffers while amortizing reads.
const BATCH_BLOCKS: usize = 16;

pub fn index_range<I: VecIndex>(first: &[I], from: usize, to: usize, len: usize) -> Range<usize> {
    first[from].to_usize()..first.get(to).map_or(len, |i| i.to_usize())
}

/// One chunk's first output and input per block, read once.
pub struct BlockBounds {
    blocks: Range<usize>,
    first_outputs: Vec<TxOutIndex>,
    first_inputs: Vec<TxInIndex>,
    output_len: usize,
    input_len: usize,
}

impl BlockBounds {
    pub fn new(indexer: &Indexer, blocks: Range<usize>) -> Self {
        let outputs = &indexer.vecs().outputs;
        let inputs = &indexer.vecs().inputs;
        let end = blocks.end + 1;
        Self {
            first_outputs: outputs
                .first_txout_index
                .collect_range_at(blocks.start, end.min(outputs.first_txout_index.len())),
            first_inputs: inputs
                .first_txin_index
                .collect_range_at(blocks.start, end.min(inputs.first_txin_index.len())),
            output_len: outputs.value.len(),
            input_len: inputs.outpoint.len(),
            blocks,
        }
    }

    fn batches(&self) -> impl Iterator<Item = Batch<'_>> {
        self.blocks.clone().step_by(BATCH_BLOCKS).map(|from| {
            let blocks = from..(from + BATCH_BLOCKS).min(self.blocks.end);
            Batch {
                outputs: self.outputs(blocks.clone()),
                inputs: self.inputs(blocks.clone()),
                bounds: self,
                blocks,
            }
        })
    }

    fn outputs(&self, blocks: Range<usize>) -> Range<usize> {
        let start = self.blocks.start;
        index_range(
            &self.first_outputs,
            blocks.start - start,
            blocks.end - start,
            self.output_len,
        )
    }

    fn inputs(&self, blocks: Range<usize>) -> Range<usize> {
        let start = self.blocks.start;
        index_range(
            &self.first_inputs,
            blocks.start - start,
            blocks.end - start,
            self.input_len,
        )
    }
}

/// Up to [`BATCH_BLOCKS`] consecutive blocks and their output and input ranges.
pub struct Batch<'a> {
    bounds: &'a BlockBounds,
    pub blocks: Range<usize>,
    outputs: Range<usize>,
    inputs: Range<usize>,
}

impl Batch<'_> {
    /// Block `height`'s outputs, as indexes into this batch's output columns.
    pub fn block_outputs(&self, height: usize) -> Range<usize> {
        let outputs = self.bounds.outputs(height..height + 1);
        outputs.start - self.outputs.start..outputs.end - self.outputs.start
    }

    /// Block `height`'s inputs after its coinbase, as indexes into this batch's
    /// input columns.
    pub fn block_spends(&self, height: usize) -> Range<usize> {
        let inputs = self.bounds.inputs(height..height + 1);
        inputs.start + 1 - self.inputs.start..inputs.end - self.inputs.start
    }
}

/// One batch's output and input columns: type indexes only for an address reader, unspendable
/// flags only for the others.
#[derive(Default)]
pub struct BatchColumns {
    pub output_values: Vec<Sats>,
    pub output_types: Vec<OutputType>,
    /// Core's coin-pruning rule per output: an unspendable one never enters the UTXO set. Address
    /// scripts are never unspendable, so an address reader leaves it empty.
    pub output_unspendable: Vec<Boolean>,
    pub output_indexes: Vec<TypeIndex>,
    pub input_values: Vec<Sats>,
    pub input_heights: Vec<Height>,
    pub input_types: Vec<OutputType>,
    pub input_indexes: Vec<TypeIndex>,
}

/// Output and input column readers, `BY_ADDRESS` for consumers tracking addresses (they read type
/// indexes) or not (they read the unspendable flags).
pub struct ColumnReaders<'a, const BY_ADDRESS: bool> {
    indexer: &'a Indexer,
    inputs: TxInReaders<'a, BY_ADDRESS>,
}

impl<'a, const BY_ADDRESS: bool> ColumnReaders<'a, BY_ADDRESS> {
    pub fn new(indexer: &'a Indexer, output_heights: &'a HeightMap<TxOutIndex>) -> Self {
        Self {
            indexer,
            inputs: TxInReaders::new(indexer, output_heights),
        }
    }

    /// Runs `process` on the batches in order, until it breaks, reading the next batch's columns
    /// beside it (a `rayon::join`: the read is a job an idle worker steals, never a thread blocked
    /// on a channel), so reads stay off the processing path. A single batch (an ordinary 1-3
    /// block update) has nothing to overlap: it is read and processed in place.
    pub fn for_each_batch(
        &mut self,
        bounds: &BlockBounds,
        mut process: impl FnMut(&Batch<'_>, &BatchColumns) -> Result<ControlFlow<()>> + Send,
    ) -> Result<()> {
        let mut batches = bounds.batches();
        let Some(mut batch) = batches.next() else {
            return Ok(());
        };
        let (mut columns, mut next_columns) = (BatchColumns::default(), BatchColumns::default());
        self.collect(&batch, &mut columns)?;
        for next in batches {
            let (processed, read) = join(
                || process(&batch, &columns),
                || self.collect(&next, &mut next_columns),
            );
            let flow = processed?;
            read?;
            if flow.is_break() {
                return Ok(());
            }
            batch = next;
            mem::swap(&mut columns, &mut next_columns);
        }
        process(&batch, &columns).map(drop)
    }

    /// Reads `batch`'s output and input columns in parallel.
    fn collect(&mut self, batch: &Batch<'_>, columns: &mut BatchColumns) -> Result<()> {
        let BatchColumns {
            output_values,
            output_types,
            output_unspendable,
            output_indexes,
            input_values,
            input_heights,
            input_types,
            input_indexes,
        } = columns;
        let (outputs, inputs) = join(
            || {
                let (from, to) = (batch.outputs.start, batch.outputs.end);
                let outputs = &self.indexer.vecs().outputs;
                outputs.value.collect_range_into_at(from, to, output_values);
                outputs
                    .output_type
                    .collect_range_into_at(from, to, output_types);
                if BY_ADDRESS {
                    outputs
                        .type_index
                        .collect_range_into_at(from, to, output_indexes);
                } else {
                    outputs
                        .is_unspendable
                        .collect_range_into_at(from, to, output_unspendable);
                }
                let count = to - from;
                if output_values.len() != count
                    || output_types.len() != count
                    || (BY_ADDRESS && output_indexes.len() != count)
                    || (!BY_ADDRESS && output_unspendable.len() != count)
                {
                    return Err(Error::NotFound(
                        "incomplete distribution output columns".into(),
                    ));
                }
                Ok(())
            },
            || {
                self.inputs.collect_inputs(
                    batch,
                    input_values,
                    input_heights,
                    input_types,
                    input_indexes,
                )
            },
        );
        outputs.and(inputs)
    }
}
