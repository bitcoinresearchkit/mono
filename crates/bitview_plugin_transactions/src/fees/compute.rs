use std::time::{Duration, Instant};

use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::CachedSeries;
use block::Block;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Sats, StoredBool, StoredU64, TxInIndex, TxIndex};
use rayon::{join, prelude::*};
use tracing::info;
use vecdb::{AnyStoredVec, AnyVec, PcoVec, ReadableVec, VecIndex, WritableVec};

use super::{super::size, Vecs};

mod block;
mod values;

const COMPUTE_BATCH_HEIGHTS: usize = 64;

#[allow(clippy::too_many_arguments)]
pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    input_values: &PcoVec<TxInIndex, Sats>,
    mappings: &MappingsVecs,
    size_vecs: &size::Vecs,
    transfer_volume: &mut CachedSeries<Height, Sats>,
    exit: &Exit,
) -> Result<()> {
    let starting_lengths = indexer.safe_lengths();

    vecs.compute_fees(
        indexer,
        input_values,
        mappings,
        size_vecs,
        transfer_volume,
        exit,
    )?;

    let vsize_source = &size_vecs.vsize.tx_index;
    let (r1, r2) = join(
        || {
            vecs.fee.derive_from_with_skip(
                mappings,
                &starting_lengths,
                &indexer.vecs().transactions.first_tx_index,
                exit,
                1,
            )
        },
        || {
            vecs.effective_fee_rate.derive_from_with_skip_weighted(
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

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    fn compute_fees(
        &mut self,
        indexer: &Indexer,
        input_values: &PcoVec<TxInIndex, Sats>,
        mappings: &MappingsVecs,
        size_vecs: &size::Vecs,
        transfer_volume: &mut CachedSeries<Height, Sats>,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();

        let raw = indexer.vecs();
        let output_values = &raw.outputs.value;
        let dep_version = input_values.version()
            + output_values.version()
            + raw.transactions.first_txout_index.version()
            + raw.inputs.first_txin_index.version()
            + raw.outputs.first_txout_index.version()
            + size_vecs.vsize.tx_index.version()
            + raw.inputs.outpoint.version()
            + raw.transactions.first_tx_index.version()
            + raw.transactions.first_txin_index.version()
            + mappings.height.tx_index_count.version();

        self.fee
            .tx_index
            .validate_computed_version_or_reset(dep_version)?;
        self.fee_rate
            .validate_computed_version_or_reset(dep_version)?;
        self.effective_fee_rate
            .tx_index
            .validate_computed_version_or_reset(dep_version)?;
        for target in self.cpfp_flags.iter_mut() {
            target.validate_computed_version_or_reset(dep_version)?;
        }
        for target in self.count.iter_mut() {
            target.validate_computed_version_or_reset(dep_version)?;
        }
        self.coinbase_value
            .validate_computed_version_or_reset(dep_version)?;

        transfer_volume.validate_computed_version_or_reset(dep_version)?;
        let target = raw
            .transactions
            .first_txin_index
            .len()
            .min(raw.transactions.first_txout_index.len())
            .min(size_vecs.vsize.tx_index.len());
        let tx_len = self
            .fee
            .tx_index
            .len()
            .min(self.fee_rate.len())
            .min(self.effective_fee_rate.tx_index.len())
            .min(
                self.cpfp_flags
                    .iter_mut()
                    .map(|v| v.len())
                    .min()
                    .unwrap_or_default(),
            )
            .min(starting_lengths.tx_index.to_usize());
        let max_height = raw
            .transactions
            .first_tx_index
            .len()
            .min(mappings.height.tx_index_count.len())
            .min(raw.inputs.first_txin_index.len())
            .min(raw.outputs.first_txout_index.len());
        let next_height = mappings
            .tx_heights
            .resume_height(tx_len, target, max_height);
        let count_len = self
            .count
            .iter_mut()
            .map(|v| v.cumulative.height.len())
            .min()
            .unwrap_or_default()
            .min(self.coinbase_value.len())
            .min(transfer_volume.len())
            .min(starting_lengths.height.to_usize())
            .min(max_height);
        let start_height = count_len.min(next_height);
        if start_height >= max_height {
            return Ok(());
        }

        let start_tx = raw
            .transactions
            .first_tx_index
            .collect_one_at(start_height)
            .unwrap()
            .to_usize();
        self.fee
            .tx_index
            .truncate_if_needed(TxIndex::from(start_tx))?;
        self.fee_rate.truncate_if_needed(TxIndex::from(start_tx))?;
        self.effective_fee_rate
            .tx_index
            .truncate_if_needed(TxIndex::from(start_tx))?;
        for target in self.cpfp_flags.iter_mut() {
            target.truncate_if_needed_at(start_tx)?;
        }
        for target in self.count.iter_mut() {
            target.truncate_if_needed_at(start_height)?;
        }
        self.coinbase_value.truncate_if_needed_at(start_height)?;

        transfer_volume.truncate_if_needed_at(start_height)?;
        // Persist the common rewind before any new block is appended.
        self.write_fees(transfer_volume, exit)?;
        let mut cumulative_volume = transfer_volume.collect_last().unwrap_or_default();

        let mut tx_count = mappings.height.tx_index_count.cursor();
        let mut next_block_input = raw.inputs.first_txin_index.cursor();
        tx_count.advance(start_height);
        next_block_input.advance(start_height);
        let start_input = next_block_input.next().unwrap().to_usize();
        let mut next_block_output = raw.outputs.first_txout_index.cursor();
        next_block_output.advance(start_height + 1);
        let mut output_starts = Vec::new();

        // Dependencies have written their vectors before this pass. Retain the
        // decoded pages across blocks instead of reopening each compressed range.
        let mut weights = raw.transactions.weight.range_cursor_at(start_tx, target);
        let mut txin_starts = raw
            .transactions
            .first_txin_index
            .range_cursor_at(start_tx, target);
        let mut outpoints = raw
            .inputs
            .outpoint
            .range_cursor_at(start_input, raw.inputs.outpoint.len());
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
                if first_tx + n > target {
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
                    .map_or(raw.inputs.outpoint.len(), |index| index.to_usize());
                let output_end = next_block_output
                    .next()
                    .map_or(output_values.len(), |index| index.to_usize());
                if input_end > input_values.len()
                    || input_end > raw.inputs.outpoint.len()
                    || output_end > output_values.len()
                {
                    break;
                }
                raw.transactions.first_txout_index.collect_range_into_at(
                    first_tx,
                    first_tx + n,
                    &mut output_starts,
                );
                input_value.for_each(
                    input_end - block.input_begin,
                    values::sum(&block.txin_starts, input_end, &mut block.input_values),
                );
                output_values.for_each_range_at(
                    output_starts[0].to_usize(),
                    output_end,
                    values::sum(&output_starts, output_end, &mut block.output_values),
                );
                outpoints.collect_into(input_end - block.input_begin, &mut block.outpoints);

                first_tx += n;
                batch_len += 1;
            }

            if batch_len == 0 {
                break;
            }
            blocks[..batch_len].par_iter_mut().for_each(Block::compute);

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
                    self.fee.tx_index.push(fee);
                    self.fee_rate.push(fee_rate);
                    self.effective_fee_rate.tx_index.push(effective);
                    self.cpfp_flags
                        .is_cpfp_parent
                        .push(StoredBool::from(is_parent));
                    self.cpfp_flags
                        .is_cpfp_child
                        .push(StoredBool::from(is_child));
                }
                self.count
                    .cpfp_parent
                    .push_block(StoredU64::from(parent_count));
                self.count
                    .cpfp_child
                    .push_block(StoredU64::from(child_count));
                self.coinbase_value.push(block.output_values[0]);

                cumulative_volume += block.transfer_volume;
                transfer_volume.push(cumulative_volume);

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
        self.fee_rate.write()?;
        self.effective_fee_rate.tx_index.write()?;
        for target in self.cpfp_flags.iter_mut() {
            target.write()?;
        }
        for target in self.count.iter_mut() {
            target.write()?;
        }
        self.coinbase_value.write()?;
        transfer_volume.write()?;
        Ok(())
    }
}
