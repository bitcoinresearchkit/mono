use std::iter::repeat_n;

use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_exit::{Exit, ExitGuard};
use brk_types::{Height, TxInIndex, TxOutIndex};
use tracing::{info, warn};
use vecdb::{AnyStoredVec, AnyVec, Error as VecError, ReadableVec, Stamp, VecIndex, WritableVec};

use super::Vecs;

const HEIGHT_BATCH: u32 = 10_000;

pub fn compute(vecs: &mut Vecs, indexer: &Indexer, exit: &Exit) -> Result<ExitGuard> {
    let starting_lengths = indexer.safe_lengths();

    let dep_version = indexer.vecs().inputs.txout_index.version()
        + indexer.vecs().outputs.first_txout_index.version()
        + indexer.vecs().inputs.first_txin_index.version()
        + indexer.vecs().outputs.value.version();
    vecs.txin_index
        .validate_computed_version_or_reset(dep_version)?;

    let target_height = indexer.vecs().blocks.blockhash.len();
    if target_height == 0 {
        return Ok(exit.lock());
    }
    let target_height = Height::from(target_height - 1);

    // Zero means uninitialized; every checkpoint counts completed blocks.
    let starting_stamp = Stamp::from(starting_lengths.height.incremented());
    if vecs.txin_index.stamp() >= starting_stamp
        && !vecs
            .txin_index
            .rollback_before(starting_stamp)
            .is_ok_and(|stamp| stamp < starting_stamp)
    {
        warn!("Could not roll back spent outputs; rebuilding");
        vecs.txin_index.reset()?;
    }
    let min_txout_index = vecs
        .txin_index
        .len()
        .min(starting_lengths.txout_index.to_usize());

    vecs.txin_index
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

    let mut batch_start_height = min_height;
    while batch_start_height <= target_height {
        let batch_end_height = (batch_start_height + HEIGHT_BATCH).min(target_height);

        // Fill txout_index up to batch_end_height + 1
        let batch_txout_index = if batch_end_height >= target_height {
            indexer.vecs().outputs.value.len()
        } else {
            first_txout_index_data[batch_end_height.to_usize() + 1 - offset].to_usize()
        };
        // Keep the batch staged: fill_to may issue an unstamped write before
        // rollback data for a reorg truncation has been saved.
        let len = vecs.txin_index.len();
        vecs.txin_index.extend(repeat_n(
            TxInIndex::UNSPENT,
            batch_txout_index.saturating_sub(len),
        ));

        // Get txin range for this height batch
        let txin_start = first_txin_index_data[batch_start_height.to_usize() - offset].to_usize();
        let txin_end = if batch_end_height >= target_height {
            indexer.vecs().inputs.txout_index.len()
        } else {
            first_txin_index_data[batch_end_height.to_usize() + 1 - offset].to_usize()
        };

        // Appended outputs are still in memory; only stored updates need sorting.
        let mut stored_updates = Vec::new();
        let stored_len = vecs.txin_index.stored_len();
        let pushed = vecs.txin_index.pushed_mut();
        let mut j = txin_start;
        txin_index_to_txout_index
            .try_for_each_range_at(txin_start, txin_end, |txout_index: TxOutIndex| {
                if !txout_index.is_coinbase() {
                    let txin_index = TxInIndex::from(j);
                    let index = txout_index.to_usize();
                    if index < stored_len {
                        stored_updates.push((txout_index, txin_index));
                    } else {
                        *pushed.get_mut(index - stored_len).ok_or(index)? = txin_index;
                    }
                }
                j += 1;
                Ok(())
            })
            .map_err(|index| VecError::IndexTooHigh {
                index,
                len: vecs.txin_index.len(),
                name: vecs.txin_index.name().to_owned(),
            })?;

        stored_updates.sort_unstable_by_key(|(txout_index, _)| *txout_index);
        vecs.txin_index.update_many(stored_updates)?;

        if batch_end_height < target_height {
            let _lock = exit.lock();
            // Catch-up batches advance the baseline without saving rollback
            // history. Only the final update write records changes.
            vecs.txin_index.stamped_write_maybe_with_changes(
                Stamp::from(batch_end_height.incremented()),
                false,
            )?;
            vecs.txin_index.flush()?;
            info!(
                "Indexing spent outputs: {:.0}%",
                batch_end_height.to_usize() as f64 / target_height.to_usize() as f64 * 100.0
            );
        }

        batch_start_height = batch_end_height + 1_u32;
    }

    let lock = exit.lock();
    if vecs.txin_index.stamp() < Stamp::from(target_height.incremented()) {
        checkpoint(vecs, target_height)?;
    }

    Ok(lock)
}

/// Save undo before changing stored outputs, then make the batch durable.
pub(super) fn checkpoint(vecs: &mut Vecs, height: Height) -> Result<()> {
    vecs.txin_index
        .stamped_write_with_changes(Stamp::from(height.incremented()))?;
    vecs.txin_index.flush()?;
    Ok(())
}
