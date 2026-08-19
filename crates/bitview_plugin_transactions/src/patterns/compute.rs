use brk_error::Result;

use brk_indexer::Indexer;
use brk_types::{Sats, StoredBool, StoredU64, TxInIndex, TxIndex};
use vecdb::{AnyStoredVec, AnyVec, ColumnId, Exit, PcoVec, ReadableVec, VecIndex, WritableVec};

use super::{PatternId, Vecs, coinjoin::Candidate};

const WRITE_INTERVAL: usize = 10_000;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    input_values: &PcoVec<TxInIndex, Sats>,
    indexes: &bitview_plugin_indexes::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, input_values, indexes, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        input_values: &PcoVec<TxInIndex, Sats>,
        indexes: &bitview_plugin_indexes::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let features = &indexer.vecs().transaction_features;
        let version = indexes.tx_index.input_count.version()
            + indexes.tx_index.output_count.version()
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
            + indexes.height.tx_index_count.version();

        self.flags_source
            .validate_computed_version_or_reset(version)?;
        self.count
            .cumulative
            .validate_computed_version_or_reset(version)?;

        let starting_lengths = indexer.safe_lengths();
        let target_tx = indexes.tx_index.input_count.len();
        let target_height = indexes.height.tx_index_count.len();
        let tx_len = self
            .flags_source
            .len()
            .min(starting_lengths.tx_index.to_usize());
        let count_len = self
            .count
            .cumulative
            .len()
            .min(starting_lengths.height.to_usize());
        let start_height = count_len.min(next_height(indexes, tx_len, target_tx, target_height));
        if start_height >= target_height {
            return Ok(());
        }

        let first_tx = &indexer.vecs().transactions.first_tx_index;
        let start_tx = first_tx.collect_one_at(start_height).unwrap().to_usize();
        self.flags_source.truncate_if_needed_at(start_tx)?;
        self.count.truncate_if_needed_at(start_height)?;

        let first_txin = indexer
            .vecs()
            .transactions
            .first_txin_index
            .collect_one_at(start_tx)
            .unwrap()
            .to_usize();
        let first_txout = indexer
            .vecs()
            .transactions
            .first_txout_index
            .collect_one_at(start_tx)
            .unwrap()
            .to_usize();

        let mut input_count = indexes.tx_index.input_count.cursor();
        let mut output_count = indexes.tx_index.output_count.cursor();
        let mut input_value = input_values.cursor();
        let mut input_type = indexer.vecs().inputs.output_type.cursor();
        let mut input_type_index = indexer.vecs().inputs.type_index.cursor();
        let mut output_value = indexer.vecs().outputs.value.reader().cursor();
        let mut output_type = indexer.vecs().outputs.output_type.reader().cursor();
        let mut output_type_index = indexer.vecs().outputs.type_index.reader().cursor();
        let mut has_op_return = features.has_op_return.cursor();
        let mut has_inscription = features.has_inscription.cursor();
        let mut tx_count = indexes.height.tx_index_count.cursor();

        input_count.advance(start_tx);
        output_count.advance(start_tx);
        input_value.advance(first_txin);
        input_type.advance(first_txin);
        input_type_index.advance(first_txin);
        output_value.advance(first_txout);
        output_type.advance(first_txout);
        output_type_index.advance(first_txout);
        has_op_return.advance(start_tx);
        has_inscription.advance(start_tx);
        tx_count.advance(start_height);

        let mut candidate = Candidate::default();
        let mut block_start = start_tx;
        for height in start_height..target_height {
            let block_end = block_start + u64::from(tx_count.next().unwrap()) as usize;
            let mut coinjoin_count = 0;
            let mut consolidation_count = 0;
            let mut batch_payout_count = 0;

            for tx_index in block_start..block_end {
                let inputs = usize::from(input_count.next().unwrap());
                let outputs = usize::from(output_count.next().unwrap());
                let op_return = has_op_return.next().unwrap().is_true();
                let inscription = has_inscription.next().unwrap().is_true();
                let token_related = op_return || inscription;

                let consolidation = is_consolidation(inputs, outputs);
                let batch_payout = is_batch_payout(inputs, outputs, tx_index == block_start);
                let coinjoin_candidate = is_coinjoin_candidate(inputs, outputs, token_related);

                let coinjoin = if coinjoin_candidate {
                    candidate.clear();
                    for _ in 0..inputs {
                        candidate.add_input(
                            input_value.next().unwrap(),
                            input_type.next().unwrap(),
                            input_type_index.next().unwrap(),
                        );
                    }
                    for _ in 0..outputs {
                        candidate.add_output(
                            output_value.next().unwrap(),
                            output_type.next().unwrap(),
                            output_type_index.next().unwrap(),
                        );
                    }
                    candidate.is_match(inputs, outputs)
                } else {
                    input_value.advance(inputs);
                    input_type.advance(inputs);
                    input_type_index.advance(inputs);
                    output_value.advance(outputs);
                    output_type.advance(outputs);
                    output_type_index.advance(outputs);
                    false
                };

                coinjoin_count += coinjoin as u64;
                consolidation_count += consolidation as u64;
                batch_payout_count += batch_payout as u64;
                self.flags_source.push([
                    StoredBool::from(coinjoin),
                    StoredBool::from(consolidation),
                    StoredBool::from(batch_payout),
                ]);
            }

            self.count
                .push_block(PatternId::from_fn(|pattern| match pattern {
                    PatternId::Coinjoin => StoredU64::from(coinjoin_count),
                    PatternId::Consolidation => StoredU64::from(consolidation_count),
                    PatternId::BatchPayout => StoredU64::from(batch_payout_count),
                }));

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
        self.flags_source.write()?;
        self.count.write()?;
        Ok(())
    }
}

fn next_height(
    indexes: &bitview_plugin_indexes::Vecs,
    tx_len: usize,
    target_tx: usize,
    target_height: usize,
) -> usize {
    if tx_len >= target_tx {
        target_height
    } else {
        indexes
            .tx_heights
            .get_shared(TxIndex::from(tx_len))
            .unwrap()
            .to_usize()
    }
}

fn is_consolidation(inputs: usize, outputs: usize) -> bool {
    inputs >= outputs * 5
}

fn is_batch_payout(inputs: usize, outputs: usize, is_coinbase: bool) -> bool {
    !is_coinbase && outputs >= inputs * 5
}

fn is_coinjoin_candidate(inputs: usize, outputs: usize, token_related: bool) -> bool {
    inputs >= 5 && outputs >= 5 && inputs < outputs * 5 && outputs < inputs * 5 && !token_related
}

#[cfg(test)]
mod tests {
    use super::{is_batch_payout, is_coinjoin_candidate, is_consolidation};

    #[test]
    fn ratio_boundaries_match_filter_semantics() {
        assert!(is_consolidation(25, 5));
        assert!(is_batch_payout(5, 25, false));
        assert!(!is_batch_payout(1, 5, true));
        assert!(!is_coinjoin_candidate(25, 5, false));
        assert!(!is_coinjoin_candidate(5, 25, false));
        assert!(is_coinjoin_candidate(5, 5, false));
        assert!(!is_coinjoin_candidate(5, 5, true));
    }
}
