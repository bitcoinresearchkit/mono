use brk_error::Result;

use brk_indexer::Indexer;
use brk_types::{Height, TxInIndex, TxOutIndex};
use tracing::info;
use vecdb::{AnyStoredVec, AnyVec, Exit, ExitGuard, ReadableVec, Stamp, VecIndex, WritableVec};

use super::Vecs;

const HEIGHT_BATCH: u32 = 10_000;

pub fn compute(vecs: &mut Vecs, indexer: &Indexer, exit: &Exit) -> Result<ExitGuard> {
    vecs.compute(indexer, exit)
}

impl Vecs {
    fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<ExitGuard> {
        let starting_lengths = indexer.safe_lengths();

        let dep_version = indexer.vecs().inputs.txout_index.version()
            + indexer.vecs().outputs.first_txout_index.version()
            + indexer.vecs().inputs.first_txin_index.version()
            + indexer.vecs().outputs.value.version();
        self.txin_index
            .validate_computed_version_or_reset(dep_version)?;

        let target_height = indexer.vecs().blocks.blockhash.len();
        if target_height == 0 {
            return Ok(exit.lock());
        }
        let target_height = Height::from(target_height - 1);

        // Find min_height from current vec length
        let current_txout_index = self.txin_index.len();
        let min_txout_index = current_txout_index.min(starting_lengths.txout_index.to_usize());

        let starting_stamp = Stamp::from(starting_lengths.height);
        let _ = self.txin_index.rollback_before(starting_stamp);

        self.txin_index
            .truncate_if_needed(TxOutIndex::from(min_txout_index))?;

        let txin_index_to_txout_index = &indexer.vecs().inputs.txout_index;
        // Find min_height via binary search (first_txout_index is monotonically non-decreasing)
        let first_txout_index_vec = &indexer.vecs().outputs.first_txout_index;
        let min_height = if min_txout_index == 0 {
            Height::ZERO
        } else if min_txout_index >= starting_lengths.txout_index.to_usize() {
            starting_lengths.height
        } else {
            let mut lo = 0usize;
            let mut hi = starting_lengths.height.to_usize() + 1;
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                if first_txout_index_vec
                    .collect_one_at(mid)
                    .unwrap()
                    .to_usize()
                    <= min_txout_index
                {
                    lo = mid + 1;
                } else {
                    hi = mid;
                }
            }
            Height::from(lo.saturating_sub(1))
        };

        // Only collect from min_height onward (not from 0)
        let offset = min_height.to_usize();
        let first_txout_index_data =
            first_txout_index_vec.collect_range_at(offset, target_height.to_usize() + 1);
        let first_txin_index_data = indexer
            .vecs()
            .inputs
            .first_txin_index
            .collect_range_at(offset, target_height.to_usize() + 2);

        debug_assert!(
            min_height <= starting_lengths.height,
            "txouts min_height ({}) exceeds starting_lengths.height ({})",
            min_height,
            starting_lengths.height
        );

        let mut pairs: Vec<(TxOutIndex, TxInIndex)> = Vec::new();

        let mut batch_start_height = min_height;
        while batch_start_height <= target_height {
            let batch_end_height = (batch_start_height + HEIGHT_BATCH).min(target_height);

            // Fill txout_index up to batch_end_height + 1
            let batch_txout_index = if batch_end_height >= target_height {
                indexer.vecs().outputs.value.len()
            } else {
                first_txout_index_data[batch_end_height.to_usize() + 1 - offset].to_usize()
            };
            self.txin_index
                .fill_to(batch_txout_index, TxInIndex::UNSPENT)?;

            // Get txin range for this height batch
            let txin_start =
                first_txin_index_data[batch_start_height.to_usize() - offset].to_usize();
            let txin_end = if batch_end_height >= target_height {
                indexer.vecs().inputs.txout_index.len()
            } else {
                first_txin_index_data[batch_end_height.to_usize() + 1 - offset].to_usize()
            };

            // Stream txins directly into pairs — avoids intermediate Vec allocation
            pairs.clear();
            let mut j = txin_start;
            txin_index_to_txout_index.for_each_range_at(
                txin_start,
                txin_end,
                |txout_index: TxOutIndex| {
                    if !txout_index.is_coinbase() {
                        pairs.push((txout_index, TxInIndex::from(j)));
                    }
                    j += 1;
                },
            );

            pairs.sort_unstable_by_key(|(txout_index, _)| *txout_index);

            for &(txout_index, txin_index) in &pairs {
                self.txin_index.update(txout_index, txin_index)?;
            }

            if batch_end_height < target_height {
                let _lock = exit.lock();
                self.txin_index.write()?;
                info!(
                    "TxOuts: {:.2}%",
                    batch_end_height.to_usize() as f64 / target_height.to_usize() as f64 * 100.0
                );
            }

            batch_start_height = batch_end_height + 1_u32;
        }

        let lock = exit.lock();
        self.txin_index
            .stamped_write_with_changes(Stamp::from(target_height))?;

        Ok(lock)
    }
}
