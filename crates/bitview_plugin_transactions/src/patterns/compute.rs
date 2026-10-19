use std::ops::Range;

use bitview_compute::{WRITE_INTERVAL, write_locked};
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Boolean, Count};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;
use rayon::prelude::*;
use vecdb::{AnyVec, ReadableVec, VecIndex, WritableVec};

use super::{Vecs, coinjoin::Candidate};
use crate::resume::prepare_blocks;

/// Heights one parallel task classifies.
const TASK_HEIGHTS: usize = 64;
/// Heights per wave of tasks, bounding the flags held before they are pushed in order.
const WAVE_HEIGHTS: usize = 1_024;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        exit: &Exit,
    ) -> Result<()> {
        let features = &indexer.vecs().transactions.features;
        let version = mappings.tx_index.input_count.version()
            + mappings.tx_index.output_count.version()
            + indexer.vecs().transactions.first_tx_index.version()
            + indexer.vecs().transactions.first_txin_index.version()
            + indexer.vecs().transactions.first_txout_index.version()
            + indexer.vecs().inputs.value.version()
            + indexer.vecs().inputs.output_type.version()
            + indexer.vecs().inputs.type_index.version()
            + indexer.vecs().outputs.value.version()
            + indexer.vecs().outputs.output_type.version()
            + indexer.vecs().outputs.type_index.version()
            + features.op_return.flag.version()
            + features.inscription.flag.version()
            + mappings.height.tx_index_count.version();

        let first_tx = &indexer.vecs().transactions.first_tx_index;
        let tx_counts = &mappings.height.tx_index_count;
        let (mut flags, mut counts) = self.stored_mut();
        let (heights, _) = prepare_blocks(
            &mut flags,
            &mut counts,
            version,
            indexer.compute_start(),
            mappings.tx_index.input_count.len(),
            tx_counts.len(),
            mappings,
            first_tx,
            exit,
        )?;

        let mut height = heights.start;
        while height < heights.end {
            let wave_end = (height + WAVE_HEIGHTS).min(heights.end);
            let tasks: Vec<Classified> = (height..wave_end)
                .into_par_iter()
                .step_by(TASK_HEIGHTS)
                .map(|from| {
                    classify(
                        indexer,
                        tx_counts,
                        from..(from + TASK_HEIGHTS).min(wave_end),
                    )
                })
                .collect();
            for task in tasks {
                for [coinjoin, consolidation, batch_payout] in task.flags {
                    self.coinjoin.flag.push(Boolean::from(coinjoin));
                    self.consolidation.flag.push(Boolean::from(consolidation));
                    self.batch_payout.flag.push(Boolean::from(batch_payout));
                }
                for [coinjoin, consolidation, batch_payout] in task.counts {
                    self.coinjoin.count.push_block(Count::from(coinjoin));
                    self.consolidation
                        .count
                        .push_block(Count::from(consolidation));
                    self.batch_payout
                        .count
                        .push_block(Count::from(batch_payout));
                }
            }
            // Writes when the wave crosses a multiple of `WRITE_INTERVAL`.
            if wave_end / WRITE_INTERVAL > height / WRITE_INTERVAL {
                self.write(exit)?;
            }
            height = wave_end;
        }

        self.write(exit)
    }

    fn write(&mut self, exit: &Exit) -> Result<()> {
        let (flags, counts) = self.stored_mut();
        write_locked(flags.into_iter().chain(counts), exit)
    }
}

/// One task's classified transactions (coinjoin, consolidation, batch payout) and per-block counts.
struct Classified {
    flags: Vec<[bool; 3]>,
    counts: Vec<[u64; 3]>,
}

/// Classifies the transactions of `heights`, independently of other tasks.
fn classify(
    indexer: &Indexer,
    tx_counts: &(impl ReadableVec<Height, Count> + Sync),
    heights: Range<usize>,
) -> Classified {
    let vecs = indexer.vecs();
    let features = &vecs.transactions.features;
    let block_tx_counts = tx_counts.collect_range_at(heights.start, heights.end);
    let start_tx = vecs
        .transactions
        .first_tx_index
        .collect_one_at(heights.start)
        .unwrap()
        .to_usize();
    let end_tx = start_tx
        + block_tx_counts
            .iter()
            .map(|&n| u64::from(n) as usize)
            .sum::<usize>();
    let input_len = vecs.inputs.outpoint.len();
    let output_len = vecs.outputs.value.len();
    // One past the task's last transaction: the next one's first input and output, when it exists.
    let txin_starts = vecs
        .transactions
        .first_txin_index
        .collect_range_at(start_tx, end_tx + 1);
    let txout_starts = vecs
        .transactions
        .first_txout_index
        .collect_range_at(start_tx, end_tx + 1);
    let has_op_return = features.op_return.flag.collect_range_at(start_tx, end_tx);
    let has_inscription = features.inscription.flag.collect_range_at(start_tx, end_tx);

    let mut input_value = vecs.inputs.value.cursor();
    let mut input_type = vecs.inputs.output_type.cursor();
    let mut input_type_index = vecs.inputs.type_index.cursor();
    let output_value = vecs.outputs.value.reader();
    let output_type = vecs.outputs.output_type.reader();
    let output_type_index = vecs.outputs.type_index.reader();

    let mut candidate = Candidate::default();
    let mut flags = Vec::with_capacity(end_tx - start_tx);
    let mut counts = Vec::with_capacity(block_tx_counts.len());
    let mut offset = 0;
    for tx_count in block_tx_counts {
        let block_start = offset;
        let block_end = offset + u64::from(tx_count) as usize;
        let mut totals = [0u64; 3];
        for tx in block_start..block_end {
            let first_txin = txin_starts[tx].to_usize();
            let first_txout = txout_starts[tx].get().to_usize();
            let next_txin = txin_starts.get(tx + 1).map_or(input_len, |i| i.to_usize());
            let next_txout = txout_starts
                .get(tx + 1)
                .map_or(output_len, |i| i.get().to_usize());
            let inputs = next_txin - first_txin;
            let outputs = next_txout - first_txout;
            let consolidation = inputs >= outputs * 5;
            let batch_payout = tx != block_start && outputs >= inputs * 5;
            let coinjoin_candidate = inputs >= 5
                && outputs >= 5
                && inputs < outputs * 5
                && outputs < inputs * 5
                && !has_op_return[tx].is_true()
                && !has_inscription[tx].is_true();
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
            totals[0] += u64::from(coinjoin);
            totals[1] += u64::from(consolidation);
            totals[2] += u64::from(batch_payout);
            flags.push([coinjoin, consolidation, batch_payout]);
        }
        counts.push(totals);
        offset = block_end;
    }
    Classified { flags, counts }
}
