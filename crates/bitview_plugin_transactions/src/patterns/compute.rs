use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Sats, StoredBool, StoredU64, TxInIndex};
use vecdb::{AnyStoredVec, AnyVec, PcoVec, ReadableVec, VecIndex, WritableVec};

use super::{Vecs, coinjoin::Candidate};

const WRITE_INTERVAL: usize = 10_000;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        input_values: &PcoVec<TxInIndex, Sats>,
        mappings: &MappingsVecs,
        exit: &Exit,
    ) -> Result<()> {
        let features = &indexer.vecs().transaction_features;
        let version = mappings.tx_index.input_count.version()
            + mappings.tx_index.output_count.version()
            + indexer.vecs().transactions.first_tx_index.version()
            + indexer.vecs().transactions.first_txin_index.version()
            + indexer.vecs().transactions.first_txout_index.version()
            + input_values.version()
            + indexer.vecs().inputs.output_type.version()
            + indexer.vecs().inputs.type_index.version()
            + indexer.vecs().outputs.value.version()
            + indexer.vecs().outputs.output_type.version()
            + indexer.vecs().outputs.type_index.version()
            + features.has_op_return.version()
            + features.has_inscription.version()
            + mappings.height.tx_index_count.version();

        {
            let _lock = exit.lock();
            for target in self.flags.iter_mut() {
                target.validate_computed_version_or_reset(version)?;
            }
            for target in self.count.iter_mut() {
                target
                    .cumulative
                    .height
                    .validate_computed_version_or_reset(version)?;
            }
        }
        let starting_lengths = indexer.safe_lengths();
        let target_tx = mappings.tx_index.input_count.len();
        let target_height = mappings.height.tx_index_count.len();
        let tx_len = self
            .flags
            .iter_mut()
            .map(|v| v.len())
            .min()
            .unwrap_or_default()
            .min(starting_lengths.tx_index.to_usize());
        let count_len = self
            .count
            .iter_mut()
            .map(|v| v.cumulative.height.len())
            .min()
            .unwrap_or_default()
            .min(starting_lengths.height.to_usize());
        let start_height = count_len.min(mappings.tx_heights.resume_height(
            tx_len,
            target_tx,
            target_height,
        ));
        if start_height >= target_height {
            return Ok(());
        }

        let first_tx = &indexer.vecs().transactions.first_tx_index;
        let start_tx = first_tx.collect_one_at(start_height).unwrap().to_usize();
        {
            let _lock = exit.lock();
            for target in self.flags.iter_mut() {
                target.truncate_if_needed_at(start_tx)?;
            }
            for target in self.count.iter_mut() {
                target
                    .cumulative
                    .height
                    .truncate_if_needed_at(start_height)?;
            }

            // The same boundaries give us both counts and candidate detail ranges.
        }
        let mut txin_starts = indexer
            .vecs()
            .transactions
            .first_txin_index
            .range_cursor_at(start_tx, target_tx);
        let mut txout_starts = indexer
            .vecs()
            .transactions
            .first_txout_index
            .range_cursor_at(start_tx, target_tx);
        let mut first_txin = txin_starts.next().unwrap().to_usize();
        let mut first_txout = txout_starts.next().unwrap().to_usize();
        let input_len = indexer.vecs().inputs.outpoint.len();
        let output_len = indexer.vecs().outputs.value.len();

        let mut input_value = input_values.cursor();
        let mut input_type = indexer.vecs().inputs.output_type.cursor();
        let mut input_type_index = indexer.vecs().inputs.type_index.cursor();
        let output_value = indexer.vecs().outputs.value.reader();
        let output_type = indexer.vecs().outputs.output_type.reader();
        let output_type_index = indexer.vecs().outputs.type_index.reader();
        let mut has_op_return = features.has_op_return.cursor();
        let mut has_inscription = features.has_inscription.cursor();
        let mut tx_count = mappings.height.tx_index_count.cursor();

        tx_count.advance(start_height);

        let mut candidate = Candidate::default();
        let mut block_start = start_tx;
        for height in start_height..target_height {
            let block_end = block_start + u64::from(tx_count.next().unwrap()) as usize;
            let mut coinjoin_count = 0;
            let mut consolidation_count = 0;
            let mut batch_payout_count = 0;

            for tx_index in block_start..block_end {
                let next_txin = txin_starts
                    .next()
                    .map_or(input_len, |index| index.to_usize());
                let next_txout = txout_starts
                    .next()
                    .map_or(output_len, |index| index.to_usize());
                let inputs = next_txin.saturating_sub(first_txin);
                let outputs = next_txout.saturating_sub(first_txout);
                let consolidation = is_consolidation(inputs, outputs);
                let batch_payout = is_batch_payout(inputs, outputs, tx_index == block_start);
                let coinjoin_candidate = is_coinjoin_candidate(inputs, outputs)
                    && !has_op_return.get(tx_index).unwrap().is_true()
                    && !has_inscription.get(tx_index).unwrap().is_true();
                let coinjoin = coinjoin_candidate && {
                    candidate.clear((inputs + outputs) / 2);
                    (first_txin..next_txin).all(|index| {
                        candidate.add(
                            input_value.get(index).unwrap(),
                            input_type.get(index).unwrap(),
                            input_type_index.get(index).unwrap(),
                        )
                    }) && (first_txout..next_txout).all(|index| {
                        candidate.add(
                            output_value.get_at(index),
                            output_type.get_at(index),
                            output_type_index.get_at(index),
                        )
                    })
                };
                first_txin = next_txin;
                first_txout = next_txout;

                coinjoin_count += coinjoin as u64;
                consolidation_count += consolidation as u64;
                batch_payout_count += batch_payout as u64;
                self.flags.is_coinjoin.push(StoredBool::from(coinjoin));
                self.flags
                    .is_consolidation
                    .push(StoredBool::from(consolidation));
                self.flags
                    .is_batch_payout
                    .push(StoredBool::from(batch_payout));
            }

            self.count
                .coinjoin
                .push_block(StoredU64::from(coinjoin_count));
            self.count
                .consolidation
                .push_block(StoredU64::from(consolidation_count));
            self.count
                .batch_payout
                .push_block(StoredU64::from(batch_payout_count));

            if (height + 1).is_multiple_of(WRITE_INTERVAL) {
                let _lock = exit.lock();
                self.write()?;
            }

            block_start = block_end;
        }

        let _lock = exit.lock();
        self.write()
    }

    fn write(&mut self) -> Result<()> {
        for target in self.flags.iter_mut() {
            target.write()?;
        }
        for target in self.count.iter_mut() {
            target.cumulative.height.write()?;
        }
        Ok(())
    }
}

fn is_consolidation(inputs: usize, outputs: usize) -> bool {
    inputs >= outputs * 5
}

fn is_batch_payout(inputs: usize, outputs: usize, is_coinbase: bool) -> bool {
    !is_coinbase && outputs >= inputs * 5
}

fn is_coinjoin_candidate(inputs: usize, outputs: usize) -> bool {
    inputs >= 5 && outputs >= 5 && inputs < outputs * 5 && outputs < inputs * 5
}
