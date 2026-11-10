mod block;
mod kinds;
mod values;

use std::time::{Duration, Instant};

use bitview_compute::write_locked;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Boolean, Count, ExpectedCount};
use bitview_traversable::Traversable;
use bitview_vecs::CachedSeries;
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use brk_types::{Height, Sats, SigOps, VSize};
use rayon::prelude::*;
use tracing::info;
use vecdb::{AnyStoredVec, AnyVec, ComputeVec, ReadableVec, VecIndex, WritableVec};

use super::Vecs;
use crate::{distribution::TxDistribution, resume::prepare_blocks};
use block::{Block, Components, is};
use kinds::{
    CPFP_CHILD, CPFP_PARENT, Columns, NONSTANDARD, ONE_INPUT, ONE_OUTPUT, READ, SEGWIT, VERSIONS,
};
pub(crate) use kinds::{Tallied, Valued};

const COMPUTE_BATCH_HEIGHTS: usize = 64;

/// The chance that `inputs` and `outputs` in random order are sorted: one over each's factorial,
/// the outputs taken as distinct. Past 20 of either it is below 1 in 21!, and counted as none:
/// under 10^-10 transactions over all of mainnet's. Each transaction looks it up rather than
/// dividing.
#[inline]
fn bip69_chance(inputs: usize, outputs: usize) -> f64 {
    const CHANCES: [[f64; 21]; 21] = {
        let mut factorials = [1.0; 21];
        let mut n = 1;
        while n < 21 {
            factorials[n] = factorials[n - 1] * n as f64;
            n += 1;
        }
        let mut table = [[0.0; 21]; 21];
        let mut i = 0;
        while i < 21 {
            let mut o = 0;
            while o < 21 {
                table[i][o] = 1.0 / (factorials[i] * factorials[o]);
                o += 1;
            }
            i += 1;
        }
        table
    };
    CHANCES
        .get(inputs)
        .and_then(|row| row.get(outputs))
        .copied()
        .unwrap_or(0.0)
}

/// Blocks between flushes, below the shared `WRITE_INTERVAL`: six per-transaction columns hold
/// their pushed rows in memory until written (~3,000 transactions per block at the tip).
const FLUSH_BLOCKS: usize = 1_000;

/// A batch of blocks read: its blocks (as many as the largest batch; an update of a few blocks
/// fills a few), each one's transactions, its first height, and its transactions' flags.
#[derive(Default)]
struct Batch {
    blocks: Vec<Block>,
    txs: Vec<usize>,
    height: usize,
    bits: [Vec<u64>; READ],
}

impl Batch {
    /// Computes the batch's blocks, in parallel.
    fn compute(&mut self, indexer: &Indexer) -> Result<()> {
        let bits = &self.bits;
        self.blocks[..self.txs.len()]
            .par_iter_mut()
            // Component scratch per worker task, freed with the batch: a big
            // component's buffers don't stay retained per block slot.
            .try_for_each_init(Components::default, |components, block| {
                block.compute(components, bits, indexer)
            })
    }
}

impl Vecs {
    /// The fee pass: the one walk of every transaction with its inputs' values, so all that needs
    /// them is computed in it: the per-block fee, effective fee rate and `vsize` distributions,
    /// every kind's value (`valued`, and its CPFP roles'), the counts it tallies (payments,
    /// transaction shapes, ephemeral dust), BIP-69's chance matches, nonstandard transactions and
    /// inscriptions' fees; then the cumulative fee total.
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        vsize: &mut TxDistribution<VSize>,
        transfer_volume: &mut CachedSeries<Height, Sats>,
        valued: &mut Valued<'_>,
        exit: &Exit,
    ) -> Result<()> {
        self.compute_fees(indexer, mappings, vsize, transfer_volume, valued, exit)?;
        self.total_cumulative.compute_cumulative(
            indexer.compute_start().height,
            &self.total,
            exit,
        )?;
        Ok(())
    }

    /// The fee pass's outputs, one restart group: per transaction, then per block.
    fn outputs_mut<'a>(
        &'a mut self,
        vsize: &'a mut TxDistribution<VSize>,
        transfer_volume: &'a mut CachedSeries<Height, Sats>,
        valued: &'a mut Valued<'_>,
    ) -> ([&'a mut dyn AnyStoredVec; 6], Vec<&'a mut dyn AnyStoredVec>) {
        let (parent_flag, [parent_count, parent_value]) = self.cpfp_parent.stored_mut();
        let (child_flag, [child_count, child_value]) = self.cpfp_child.stored_mut();
        let (nonstandard_flag, valued_blocks) = valued.stored_mut();
        let mut blocks: Vec<&mut dyn AnyStoredVec> = vec![
            &mut self.coinbase_value,
            transfer_volume,
            &mut self.total,
            parent_count,
            child_count,
            parent_value,
            child_value,
        ];
        blocks.extend(valued_blocks);
        blocks.extend(self.fee.distribution.iter_any_stored_mut());
        blocks.extend(self.effective_fee_rate.distribution.iter_any_stored_mut());
        blocks.extend(vsize.iter_any_stored_mut());
        blocks.extend(self.ranges.stored_mut());
        (
            [
                &mut self.fee.tx,
                &mut self.fee_rate,
                &mut self.effective_fee_rate.tx,
                parent_flag,
                child_flag,
                nonstandard_flag,
            ],
            blocks,
        )
    }

    fn compute_fees(
        &mut self,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        vsize: &mut TxDistribution<VSize>,
        transfer_volume: &mut CachedSeries<Height, Sats>,
        valued: &mut Valued<'_>,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.compute_start();
        let vecs = indexer.vecs();
        let columns = Columns::new(indexer);
        let (txs, inputs, outputs) = (&vecs.transactions, &vecs.inputs, &vecs.outputs);
        let (input_values, output_values, outpoints) =
            (&inputs.value, &outputs.value, &inputs.outpoint);
        let tx_counts = &mappings.height.tx_index_count;
        let version = input_values.version()
            + output_values.version()
            + txs.first_txout_index.version()
            + inputs.first_txin_index.version()
            + outputs.first_txout_index.version()
            + txs.first_tx_index.version()
            + txs.first_txin_index.version()
            + tx_counts.version()
            + txs.weight.version()
            + txs.total_sigop_cost.version()
            + txs.total_size.version()
            + txs.tx_version.version()
            + columns.version()
            + vecs.op_return.first_index.version()
            + vecs.op_return.to_tx_index.version()
            + outputs.output_type.version()
            + inputs.output_type.version()
            + outpoints.version();
        let target = txs.first_txin_index.len();
        let max_height = txs.first_tx_index.len();
        let (heights, start_tx) = {
            let (mut txs_out, mut blocks_out) = self.outputs_mut(vsize, transfer_volume, valued);
            prepare_blocks(
                &mut txs_out,
                &mut blocks_out,
                version,
                starting_lengths,
                target,
                max_height,
                mappings,
                &txs.first_tx_index,
                exit,
            )?
        };
        let start_height = heights.start;
        if heights.is_empty() {
            return Ok(());
        }

        let mut cumulative = (
            transfer_volume.collect_last().unwrap_or_default(),
            valued.inscription_fees.collect_last().unwrap_or_default(),
        );

        // Bounded: an update of a few blocks reads only their counts and boundaries.
        let mut tx_count = tx_counts.bounded_cursor_at(start_height, max_height);
        let mut next_block_input = inputs
            .first_txin_index
            .bounded_cursor_at(start_height, max_height + 1);
        let start_input = next_block_input.next().unwrap().to_usize();
        let mut next_block_output = outputs
            .first_txout_index
            .bounded_cursor_at(start_height + 1, max_height + 1);

        // Dependencies have written their vectors before this pass. Retain the
        // decoded pages across blocks instead of reopening each compressed range.
        let mut weights = txs.weight.range_cursor_at(start_tx, target);
        let mut sigops = txs.total_sigop_cost.range_cursor_at(start_tx, target);
        let mut sizes = txs.total_size.range_cursor_at(start_tx, target);
        let mut tx_versions = txs.tx_version.range_cursor_at(start_tx, target);
        let mut txin_starts = txs.first_txin_index.range_cursor_at(start_tx, target);
        // Ascending, so walked alongside the transactions.
        let op_returns = &vecs.op_return;
        let mut op_return_txs = op_returns.to_tx_index.range_cursor_at(
            op_returns
                .first_index
                .collect_one_at(start_height)
                .data()?
                .to_usize(),
            op_returns.to_tx_index.len(),
        );
        let mut next_op_return_tx = op_return_txs.next();
        let outpoint_len = outpoints.len();
        let mut outpoints = outpoints.range_cursor_at(start_input, outpoint_len);
        let mut input_value = input_values.range_cursor_at(start_input, input_values.len());

        let mut scratch_sizes = Vec::new();
        let mut scratch_sigops = Vec::new();
        let mut output_starts = Vec::new();
        let mut first_tx = start_tx;
        let mut height = start_height;
        let mut last_progress = Instant::now();
        // The batch being read, and the one read before it, computed and pushed meanwhile.
        let mut reading = Batch::default();
        let mut read: Option<Batch> = None;

        loop {
            let more = height < max_height;
            if !more && read.is_none() {
                break;
            }
            let mut pushed = Ok(());
            rayon::in_place_scope(|scope| {
                if let Some(batch) = &mut read {
                    let (this, vsize, transfer_volume, valued, cumulative, pushed) = (
                        &mut *self,
                        &mut *vsize,
                        &mut *transfer_volume,
                        &mut *valued,
                        &mut cumulative,
                        &mut pushed,
                    );
                    scope.spawn(move |_| {
                        *pushed = batch.compute(indexer).and_then(|()| {
                            this.push_batch(batch, vsize, transfer_volume, valued, cumulative, exit)
                        });
                    });
                }
                if !more {
                    return;
                }
                let Batch {
                    blocks,
                    txs: batch_txs,
                    height: batch_height,
                    bits,
                } = &mut reading;
                *batch_height = height;
                batch_txs.clear();
                batch_txs.extend(
                    (height..(height + COMPUTE_BATCH_HEIGHTS).min(max_height))
                        .map(|_| u64::from(tx_count.next().unwrap()) as usize),
                );
                // Its flags decode on other threads while this one reads the rest.
                let (columns, batch_txs, batch_first_tx) = (&columns, &*batch_txs, first_tx);
                scope.spawn(move |_| {
                    columns.read(
                        height..height + batch_txs.len(),
                        batch_txs,
                        batch_first_tx,
                        bits,
                    )
                });
                for (offset, &n) in batch_txs.iter().enumerate() {
                    if offset == blocks.len() {
                        blocks.push(Block::default());
                    }
                    let block = &mut blocks[offset];
                    block.reset(first_tx);
                    block.height = height + offset;
                    block.batch_offset = first_tx - batch_first_tx;

                    // SegWit serialization weighs its witness bytes once, not four times.
                    block.vsizes.reserve(n);
                    block.weights.reserve(n);
                    block.masks.reserve(n);
                    sizes.collect_into(n, &mut scratch_sizes);
                    sigops.collect_into(n, &mut scratch_sigops);
                    let mut size = scratch_sizes.iter();
                    let mut sigop = scratch_sigops.iter();
                    weights.for_each(n, |weight| {
                        block.vsizes.push(weight.into());
                        block.weights.push(
                            SigOps::from(*sigop.next().unwrap()).adjust_weight(weight.into()),
                        );
                        let size = u64::from(u32::from(*size.next().unwrap()));
                        block
                            .masks
                            .push(u64::from(u64::from(weight) != 4 * size) << SEGWIT);
                    });
                    let mut mask = block.masks.iter_mut();
                    tx_versions.for_each(n, |version| {
                        // 1, 2, 3, then any other (the indexer's 255).
                        let kind = (*version as usize).wrapping_sub(1).min(3);
                        *mask.next().unwrap() |= 1 << (VERSIONS + kind);
                    });
                    txin_starts.collect_into(n, &mut block.txin_starts);
                    block.input_begin = block.txin_starts[0].to_usize();
                    let input_end = next_block_input
                        .next()
                        .map_or(outpoint_len, |index| index.to_usize());
                    let output_end = next_block_output
                        .next()
                        .map_or(output_values.len(), |index| index.to_usize());
                    txs.first_txout_index.collect_range_into_at(
                        first_tx,
                        first_tx + n,
                        &mut output_starts,
                    );
                    for (tx, mask) in block.masks.iter_mut().enumerate() {
                        let first_input = block.txin_starts[tx].to_usize();
                        let inputs = block
                            .txin_starts
                            .get(tx + 1)
                            .map_or(input_end, |i| i.to_usize())
                            - first_input;
                        let tx_outputs = output_starts
                            .get(tx + 1)
                            .map_or(output_end, |i| i.get().to_usize())
                            - output_starts[tx].get().to_usize();
                        *mask |= (u64::from(inputs == 1) << ONE_INPUT)
                            | (u64::from(tx_outputs == 1) << ONE_OUTPUT);
                        block.bip69_expected += bip69_chance(inputs, tx_outputs);
                        block.tally.one_input_one_output +=
                            u64::from(inputs == 1 && tx_outputs == 1);
                        block.tally.one_input_two_output +=
                            u64::from(inputs == 1 && tx_outputs == 2);
                        let mut paying = tx_outputs;
                        while next_op_return_tx
                            .is_some_and(|op_tx| op_tx.to_usize() == first_tx + tx)
                        {
                            paying -= 1;
                            next_op_return_tx = op_return_txs.next();
                        }
                        // All but one, its likely change, when there are several.
                        block.payments.push((paying - usize::from(paying > 1)) as u32);
                    }
                    input_value.for_each(
                        input_end - block.input_begin,
                        values::sum(&block.txin_starts, input_end, &mut block.input_values),
                    );
                    output_values.for_each_range_at(
                        output_starts[0].get().to_usize(),
                        output_end,
                        values::sum(&output_starts, output_end, &mut block.output_values),
                    );
                    outpoints.collect_into(input_end - block.input_begin, &mut block.outpoints);
                    first_tx += n;
                }
            });
            pushed?;

            // The batch read is pushed next; the one pushed is read into.
            let done = read.take();
            if more {
                height += reading.txs.len();
                read = Some(std::mem::replace(&mut reading, done.unwrap_or_default()));
                if last_progress.elapsed() >= Duration::from_secs(10) {
                    info!(
                        "Computing transaction fees: block {}/{}, transactions {}/{}",
                        height - 1,
                        max_height - 1,
                        first_tx - start_tx,
                        target - start_tx,
                    );
                    last_progress = Instant::now();
                }
            }
        }

        self.write_fees(vsize, transfer_volume, valued, exit)
    }

    /// Pushes a batch computed, block by block: `cumulative` the transfer volume and inscriptions'
    /// fees so far.
    fn push_batch(
        &mut self,
        batch: &mut Batch,
        vsize: &mut TxDistribution<VSize>,
        transfer_volume: &mut CachedSeries<Height, Sats>,
        valued: &mut Valued<'_>,
        cumulative: &mut (Sats, Sats),
        exit: &Exit,
    ) -> Result<()> {
        let Batch {
            blocks,
            txs,
            height,
            ..
        } = batch;
        for (offset, block) in blocks[..txs.len()].iter().enumerate() {
            let mut parent_count = 0;
            let mut child_count = 0;
            let mut nonstandard_count = 0;
            for (((&fee, &fee_rate), &effective), &mask) in block
                .fees
                .iter()
                .zip(&block.fee_rates)
                .zip(&block.effective_fee_rates)
                .zip(&block.masks)
            {
                let is_parent = is(mask, CPFP_PARENT);
                let is_child = is(mask, CPFP_CHILD);
                let is_nonstandard = is(mask, NONSTANDARD);
                parent_count += u64::from(is_parent);
                child_count += u64::from(is_child);
                nonstandard_count += u64::from(is_nonstandard);
                self.fee.tx.push(fee);
                self.fee_rate.push(fee_rate);
                self.effective_fee_rate.tx.push(effective);
                self.cpfp_parent.flag.push(Boolean::from(is_parent));
                self.cpfp_child.flag.push(Boolean::from(is_child));
                valued.nonstandard_flag.push(Boolean::from(is_nonstandard));
            }
            self.cpfp_parent.count.push_block(Count::from(parent_count));
            self.cpfp_child.count.push_block(Count::from(child_count));
            self.cpfp_parent.count.value.push(block.values[CPFP_PARENT]);
            self.cpfp_child.count.value.push(block.values[CPFP_CHILD]);
            valued
                .nonstandard_count
                .push_block(Count::from(nonstandard_count));
            for (bit, value) in valued.values_mut() {
                value.push(block.values[bit]);
            }
            self.ranges.push_block(&block.ranges);
            valued.tallied.push_block(&block.tally);
            valued
                .vsize_total
                .push_block(block.vsizes.iter().copied().sum());
            valued
                .bip69_expected
                .push_block(ExpectedCount::from(block.bip69_expected));
            cumulative.1 += block.inscription_fees;
            valued.inscription_fees.push(cumulative.1);
            self.fee.distribution.push(block.fee_stats);
            self.effective_fee_rate.distribution.push(block.rate_stats);
            vsize.push(block.vsize_stats);
            self.coinbase_value.push(block.output_values[0]);
            self.total.push(block.total_fee);
            cumulative.0 += block.transfer_volume;
            transfer_volume.push(cumulative.0);

            if (*height + offset + 1).is_multiple_of(FLUSH_BLOCKS) {
                self.write_fees(vsize, transfer_volume, valued, exit)?;
            }
        }
        Ok(())
    }

    fn write_fees(
        &mut self,
        vsize: &mut TxDistribution<VSize>,
        transfer_volume: &mut CachedSeries<Height, Sats>,
        valued: &mut Valued<'_>,
        exit: &Exit,
    ) -> Result<()> {
        let (txs, blocks) = self.outputs_mut(vsize, transfer_volume, valued);
        write_locked(txs.into_iter().chain(blocks), exit)
    }
}
