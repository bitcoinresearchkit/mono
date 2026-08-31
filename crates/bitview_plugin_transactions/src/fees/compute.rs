use brk_error::Result;
use rayon::prelude::*;

use bitview_plugin_indexer::Indexer;
use brk_exit::Exit;
use brk_types::{Sats, StoredBool, StoredU64, TxInIndex, TxIndex};
use vecdb::{AnyStoredVec, AnyVec, ColumnId, PcoVec, ReadableVec, VecIndex, WritableVec};

use super::super::size;
use super::{CpfpRoleId, Vecs};

mod block;

use block::Block;

const COMPUTE_BATCH_HEIGHTS: usize = 64;

#[allow(clippy::too_many_arguments)]
pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    input_values: &PcoVec<TxInIndex, Sats>,
    mappings: &bitview_plugin_mappings::Vecs,
    size_vecs: &size::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, input_values, mappings, size_vecs, exit)
}

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    fn compute(
        &mut self,
        indexer: &Indexer,
        input_values: &PcoVec<TxInIndex, Sats>,
        mappings: &bitview_plugin_mappings::Vecs,
        size_vecs: &size::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();

        self.input_value.compute_sum_from_indexes(
            starting_lengths.tx_index,
            &indexer.vecs().transactions.first_txin_index,
            &mappings.tx_index.input_count,
            input_values,
            exit,
        )?;
        self.output_value.compute_sum_from_indexes(
            starting_lengths.tx_index,
            &indexer.vecs().transactions.first_txout_index,
            &mappings.tx_index.output_count,
            &indexer.vecs().outputs.value,
            exit,
        )?;

        self.compute_fees(indexer, mappings, size_vecs, exit)?;

        let vsize_source = &size_vecs.vsize.tx_index;
        let (r1, r2) = rayon::join(
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

    fn compute_fees(
        &mut self,
        indexer: &Indexer,
        mappings: &bitview_plugin_mappings::Vecs,
        size_vecs: &size::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();

        let dep_version = self.input_value.version()
            + self.output_value.version()
            + size_vecs.vsize.tx_index.version()
            + indexer.vecs().inputs.outpoint.version()
            + indexer.vecs().transactions.first_tx_index.version()
            + indexer.vecs().transactions.first_txin_index.version()
            + mappings.height.tx_index_count.version();

        self.fee
            .tx_index
            .validate_computed_version_or_reset(dep_version)?;
        self.fee_rate
            .validate_computed_version_or_reset(dep_version)?;
        self.effective_fee_rate
            .tx_index
            .validate_computed_version_or_reset(dep_version)?;
        self.cpfp_flags_source
            .validate_computed_version_or_reset(dep_version)?;
        self.count
            .cumulative
            .validate_computed_version_or_reset(dep_version)?;

        let target = self
            .input_value
            .len()
            .min(self.output_value.len())
            .min(size_vecs.vsize.tx_index.len());
        let tx_len = self
            .fee
            .tx_index
            .len()
            .min(self.fee_rate.len())
            .min(self.effective_fee_rate.tx_index.len())
            .min(self.cpfp_flags_source.len())
            .min(starting_lengths.tx_index.to_usize());
        let max_height = indexer
            .vecs()
            .transactions
            .first_tx_index
            .len()
            .min(mappings.height.tx_index_count.len());
        let next_height = if tx_len >= target {
            max_height
        } else {
            mappings
                .tx_heights
                .get_shared(TxIndex::from(tx_len))
                .unwrap()
                .to_usize()
        };
        let count_len = self.count.cumulative.len().min(max_height);
        let start_height = count_len.min(next_height);
        if start_height >= max_height {
            return Ok(());
        }

        let start_tx = indexer
            .vecs()
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
        self.cpfp_flags_source
            .truncate_if_needed(TxIndex::from(start_tx))?;
        self.count.truncate_if_needed_at(start_height)?;

        let mut tx_count = mappings.height.tx_index_count.cursor();
        let mut next_block_input = indexer.vecs().inputs.first_txin_index.cursor();
        tx_count.advance(start_height);
        next_block_input.advance(start_height + 1);

        let mut blocks: Vec<Block> = (0..COMPUTE_BATCH_HEIGHTS)
            .map(|_| Block::default())
            .collect();
        let mut first_tx = start_tx;
        let mut height = start_height;

        while height < max_height {
            let batch_end = (height + COMPUTE_BATCH_HEIGHTS).min(max_height);
            let mut batch_len = 0;

            for h in height..batch_end {
                let n = u64::from(tx_count.next().unwrap()) as usize;
                if first_tx + n > target {
                    break;
                }
                let block = &mut blocks[batch_len];
                block.reset(first_tx);

                self.input_value.collect_range_into_at(
                    first_tx,
                    first_tx + n,
                    &mut block.input_values,
                );
                self.output_value.collect_range_into_at(
                    first_tx,
                    first_tx + n,
                    &mut block.output_values,
                );
                size_vecs.vsize.tx_index.collect_range_into_at(
                    first_tx,
                    first_tx + n,
                    &mut block.vsizes,
                );
                indexer
                    .vecs()
                    .transactions
                    .first_txin_index
                    .collect_range_into_at(first_tx, first_tx + n, &mut block.txin_starts);
                block.input_begin = block.txin_starts[0].to_usize();
                let input_end = if h + 1 < max_height {
                    next_block_input.next().unwrap().to_usize()
                } else {
                    indexer.vecs().inputs.outpoint.len()
                };
                indexer.vecs().inputs.outpoint.collect_range_into_at(
                    block.input_begin,
                    input_end,
                    &mut block.outpoints,
                );

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
                    self.cpfp_flags_source
                        .push([StoredBool::from(is_parent), StoredBool::from(is_child)]);
                }
                self.count
                    .push_block(CpfpRoleId::from_fn(|role| match role {
                        CpfpRoleId::Parent => StoredU64::from(parent_count),
                        CpfpRoleId::Child => StoredU64::from(child_count),
                    }));

                if (height + offset) % 1_000 == 0 {
                    let _lock = exit.lock();
                    self.fee.tx_index.write()?;
                    self.fee_rate.write()?;
                    self.effective_fee_rate.tx_index.write()?;
                    self.cpfp_flags_source.write()?;
                    self.count.write()?;
                }
            }

            height += batch_len;
            if height < batch_end {
                break;
            }
        }

        let _lock = exit.lock();
        self.fee.tx_index.write()?;
        self.fee_rate.write()?;
        self.effective_fee_rate.tx_index.write()?;
        self.cpfp_flags_source.write()?;
        self.count.write()?;

        Ok(())
    }
}
