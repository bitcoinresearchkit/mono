mod block;
mod sources;
mod values;

use std::time::{Duration, Instant};

use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::{HeightMap, Vecs as MappingsVecs};
use bitview_primitives::{Boolean, Count, Lengths, TxInIndex};
use bitview_vecs::CachedSeries;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Sats, TxIndex};
use rayon::{join, prelude::*};
use tracing::info;
use vecdb::{AnyStoredVec, AnyVec, Error as VecError, PcoVec, ReadableVec, VecIndex, WritableVec};

use super::{super::size, Vecs};
use block::Block;
use sources::Sources;

const COMPUTE_BATCH_HEIGHTS: usize = 64;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        input_values: &PcoVec<TxInIndex, Sats>,
        mappings: &MappingsVecs,
        size_vecs: &size::Vecs,
        transfer_volume: &mut CachedSeries<Height, Sats>,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();

        self.compute_fees(
            Sources::from(indexer),
            starting_lengths,
            input_values,
            &mappings.tx_heights.read(),
            &mappings.height.tx_index_count,
            transfer_volume,
            exit,
        )?;

        let vsize_source = &size_vecs.vsize.tx_index;
        let (r1, r2) = join(
            || {
                self.fee.derive_from_with_skip(
                    mappings,
                    &starting_lengths,
                    &indexer.vecs().transactions.first_tx_index,
                    exit,
                    1,
                )
            },
            || {
                self.effective_fee_rate.derive_from_with_skip_weighted(
                    mappings,
                    &starting_lengths,
                    &indexer.vecs().transactions.first_tx_index,
                    vsize_source,
                    exit,
                    1,
                )
            },
        );
        r1?;
        r2?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_fees(
        &mut self,
        raw: Sources<'_>,
        starting_lengths: Lengths,
        input_values: &PcoVec<TxInIndex, Sats>,
        tx_heights: &HeightMap<TxIndex>,
        tx_counts: &impl ReadableVec<Height, Count>,
        transfer_volume: &mut CachedSeries<Height, Sats>,
        exit: &Exit,
    ) -> Result<()> {
        let output_values = raw.values;
        let monetary_version = input_values.version()
            + output_values.version()
            + raw.tx_outputs.version()
            + raw.block_inputs.version()
            + raw.block_outputs.version()
            + raw.block_txs.version()
            + raw.tx_inputs.version()
            + tx_counts.version();

        let rate_version = monetary_version + raw.weights.version();
        let cpfp_version = rate_version + raw.outpoints.version();
        {
            let _lock = exit.lock();
            self.fee
                .tx_index
                .validate_computed_version_or_reset(monetary_version)?;
            self.coinbase_value
                .validate_computed_version_or_reset(monetary_version)?;
            transfer_volume.validate_computed_version_or_reset(monetary_version)?;
            self.total
                .validate_computed_version_or_reset(monetary_version)?;
            self.fee_rate
                .validate_computed_version_or_reset(rate_version)?;
            self.effective_fee_rate
                .tx_index
                .validate_computed_version_or_reset(cpfp_version)?;
            for target in self.cpfp_flags_mut() {
                target.validate_computed_version_or_reset(cpfp_version)?;
            }
            for target in self.cpfp_counts_mut() {
                target.validate_computed_version_or_reset(cpfp_version)?;
            }
        }
        let target = raw.tx_inputs.len().min(raw.tx_outputs.len());
        let available_target = target.min(raw.weights.len());
        let max_height = raw
            .block_txs
            .len()
            .min(tx_counts.len())
            .min(raw.block_inputs.len())
            .min(raw.block_outputs.len());
        let resume = |tx_len| tx_heights.resume_height(tx_len, target, max_height);
        let limit = starting_lengths.height.to_usize().min(max_height);
        let monetary_start = resume(
            self.fee
                .tx_index
                .len()
                .min(starting_lengths.tx_index.to_usize()),
        )
        .min(self.coinbase_value.len())
        .min(transfer_volume.len())
        .min(limit);
        let rate_start = resume(self.fee_rate.len()).min(limit);
        let cpfp_start = resume(
            self.effective_fee_rate.tx_index.len().min(
                self.cpfp_flags_mut()
                    .map(|v| v.len())
                    .min()
                    .unwrap_or_default(),
            ),
        )
        .min(
            self.cpfp_counts_mut()
                .map(|v| v.cumulative.height.len())
                .min()
                .unwrap_or_default(),
        )
        .min(limit);
        let start_height = monetary_start.min(rate_start).min(cpfp_start);
        let first_tx = raw.block_txs;
        let to_tx = |height| {
            first_tx
                .collect_one_at(height)
                .map_or(target, |i| i.to_usize())
        };
        {
            let _lock = exit.lock();
            self.fee
                .tx_index
                .truncate_if_needed_at(to_tx(monetary_start))?;
            self.coinbase_value.truncate_if_needed_at(monetary_start)?;
            transfer_volume.truncate_if_needed_at(monetary_start)?;
            self.fee_rate.truncate_if_needed_at(to_tx(rate_start))?;
            self.effective_fee_rate
                .tx_index
                .truncate_if_needed_at(to_tx(cpfp_start))?;
            for target in self.cpfp_flags_mut() {
                target.truncate_if_needed_at(to_tx(cpfp_start))?;
            }
            for target in self.cpfp_counts_mut() {
                target.truncate_if_needed_at(cpfp_start)?;
            }
            self.total.truncate_if_needed_at(monetary_start)?;
        }
        // A newly introduced/missing block total can be rebuilt cheaply from the
        // valid fee prefix, without invalidating monetary facts or CPFP.
        self.total.compute_batched_to(
            Height::from(monetary_start),
            monetary_start,
            monetary_version,
            20_000,
            |totals, range| {
                let starts = first_tx.collect_range_at(range.start, range.end);
                let counts = tx_counts.collect_range_at(range.start, range.end);
                let mut fees = self.fee.tx_index.cursor();
                for (&start, count) in starts.iter().zip(counts) {
                    let mut total = Sats::ZERO;
                    fees.try_for_each_range_at(
                        start.to_usize(),
                        start.to_usize() + u64::from(count) as usize,
                        |fee| {
                            total += fee;
                            Ok::<_, VecError>(())
                        },
                    )?;
                    totals.push(total);
                }
                Ok(())
            },
            exit,
        )?;
        self.write_fees(transfer_volume, exit)?;
        if start_height >= max_height {
            return Ok(());
        }

        let start_tx = raw
            .block_txs
            .collect_one_at(start_height)
            .unwrap()
            .to_usize();
        let mut cumulative_volume = transfer_volume.collect_last().unwrap_or_default();

        let mut tx_count = tx_counts.cursor();
        let mut next_block_input = raw.block_inputs.cursor();
        tx_count.advance(start_height);
        next_block_input.advance(start_height);
        let start_input = next_block_input.next().unwrap().to_usize();
        let mut next_block_output = raw.block_outputs.cursor();
        next_block_output.advance(start_height + 1);
        let mut output_starts = Vec::new();

        // Dependencies have written their vectors before this pass. Retain the
        // decoded pages across blocks instead of reopening each compressed range.
        let mut weights = raw.weights.range_cursor_at(start_tx, available_target);
        let mut txin_starts = raw.tx_inputs.range_cursor_at(start_tx, target);
        let mut outpoints = raw
            .outpoints
            .range_cursor_at(start_input, raw.outpoints.len());
        let mut input_value = input_values.range_cursor_at(start_input, input_values.len());

        let mut blocks: Vec<Block> = (0..COMPUTE_BATCH_HEIGHTS)
            .map(|_| Block::default())
            .collect();
        let mut first_tx = start_tx;
        let mut height = start_height;
        let mut last_progress = Instant::now();

        while height < max_height {
            let batch_end = (height + COMPUTE_BATCH_HEIGHTS).min(max_height);
            let mut batch_len = 0;

            for _ in height..batch_end {
                let n = u64::from(tx_count.next().unwrap()) as usize;
                if first_tx + n > available_target {
                    break;
                }
                let block = &mut blocks[batch_len];
                block.reset(first_tx);

                block.vsizes.reserve(n);
                weights.for_each(n, |weight| block.vsizes.push(weight.into()));
                txin_starts.collect_into(n, &mut block.txin_starts);
                block.input_begin = block.txin_starts[0].to_usize();
                let input_end = next_block_input
                    .next()
                    .map_or(raw.outpoints.len(), |index| index.to_usize());
                let output_end = next_block_output
                    .next()
                    .map_or(output_values.len(), |index| index.to_usize());
                if input_end > raw.outpoints.len()
                    || (height + batch_len >= monetary_start
                        && (input_end > input_values.len() || output_end > output_values.len()))
                {
                    break;
                }
                if height + batch_len >= monetary_start {
                    raw.tx_outputs.collect_range_into_at(
                        first_tx,
                        first_tx + n,
                        &mut output_starts,
                    );
                    input_value.advance(block.input_begin.saturating_sub(input_value.position()));
                    input_value.for_each(
                        input_end - block.input_begin,
                        values::sum(&block.txin_starts, input_end, &mut block.input_values),
                    );
                    output_values.for_each_range_at(
                        output_starts[0].get().to_usize(),
                        output_end,
                        values::sum(&output_starts, output_end, &mut block.output_values),
                    );
                } else {
                    self.fee.tx_index.collect_range_into_at(
                        first_tx,
                        first_tx + n,
                        &mut block.fees,
                    );
                }
                outpoints.collect_into(input_end - block.input_begin, &mut block.outpoints);

                first_tx += n;
                batch_len += 1;
            }

            if batch_len == 0 {
                break;
            }
            blocks[..batch_len]
                .par_iter_mut()
                .enumerate()
                .for_each(|(offset, block)| {
                    if height + offset >= monetary_start {
                        block.compute();
                    } else {
                        block.compute_from_fees();
                    }
                });

            for (offset, block) in blocks[..batch_len].iter().enumerate() {
                let mut parent_count = 0;
                let mut child_count = 0;
                for ((&fee, &fee_rate), &effective) in block
                    .fees
                    .iter()
                    .zip(&block.fee_rates)
                    .zip(&block.effective_fee_rates)
                {
                    let is_parent = effective > fee_rate;
                    let is_child = effective < fee_rate;
                    parent_count += u64::from(is_parent);
                    child_count += u64::from(is_child);
                    if height + offset >= monetary_start {
                        self.fee.tx_index.push(fee);
                    }
                    if height + offset >= rate_start {
                        self.fee_rate.push(fee_rate);
                    }
                    if height + offset >= cpfp_start {
                        self.effective_fee_rate.tx_index.push(effective);
                        self.cpfp_parent.flag.push(Boolean::from(is_parent));
                        self.cpfp_child.flag.push(Boolean::from(is_child));
                    }
                }
                if height + offset >= cpfp_start {
                    self.cpfp_parent.count.push_block(Count::from(parent_count));
                    self.cpfp_child.count.push_block(Count::from(child_count));
                }
                if height + offset >= monetary_start {
                    self.coinbase_value.push(block.output_values[0]);
                    self.total.push(block.total_fee);
                    cumulative_volume += block.transfer_volume;
                    transfer_volume.push(cumulative_volume);
                }

                if (height + offset) % 1_000 == 0 {
                    self.write_fees(transfer_volume, exit)?;
                }
            }

            height += batch_len;
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
            if height < batch_end {
                break;
            }
        }

        self.write_fees(transfer_volume, exit)
    }

    fn write_fees(
        &mut self,
        transfer_volume: &mut CachedSeries<Height, Sats>,
        exit: &Exit,
    ) -> Result<()> {
        let _lock = exit.lock();
        self.fee.tx_index.write()?;
        self.total.write()?;
        self.fee_rate.write()?;
        self.effective_fee_rate.tx_index.write()?;
        for target in self.cpfp_flags_mut() {
            target.write()?;
        }
        for target in self.cpfp_counts_mut() {
            target.write()?;
        }
        self.coinbase_value.write()?;
        transfer_volume.write()?;
        Ok(())
    }
}
