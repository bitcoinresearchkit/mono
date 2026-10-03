mod tx_in;
mod tx_out;
use std::ops::Range;

use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::HeightMap;
use bitview_primitives::{TxInIndex, TxOutIndex};
use brk_error::Result;
use brk_types::{Height, Sats};
use rayon::join;
use vecdb::{AnyVec, PcoVec, ReadableVec, VecIndex};

use tx_in::{InputColumns, TxInReaders};
use tx_out::{OutputColumns, TxOutReaders};

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

    pub fn batches(&self) -> impl Iterator<Item = Batch<'_>> {
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

/// Reusable output and input column readers; type indexes only `WITH_INDEXES`.
pub struct Columns<'a, const WITH_INDEXES: bool> {
    outputs: TxOutReaders<'a, WITH_INDEXES>,
    inputs: TxInReaders<'a, WITH_INDEXES>,
}

impl<'a, const WITH_INDEXES: bool> Columns<'a, WITH_INDEXES> {
    pub fn new(
        indexer: &'a Indexer,
        input_values: &'a PcoVec<TxInIndex, Sats>,
        output_heights: &'a HeightMap<TxOutIndex>,
    ) -> Self {
        Self {
            outputs: TxOutReaders::new(indexer),
            inputs: TxInReaders::new(indexer, input_values, output_heights),
        }
    }

    /// Reads `batch`'s output and input columns in parallel.
    pub fn collect(&mut self, batch: &Batch<'_>) -> Result<(OutputColumns<'_>, InputColumns<'_>)> {
        let (outputs, inputs) = join(
            || {
                self.outputs
                    .collect_outputs(batch.outputs.start, batch.outputs.len())
            },
            || {
                self.inputs.collect_inputs(
                    batch.inputs.start,
                    batch.inputs.len(),
                    Height::from(batch.blocks.start),
                )
            },
        );
        Ok((outputs?, inputs?))
    }
}
