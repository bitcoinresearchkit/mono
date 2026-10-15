use std::iter::repeat_n;

use bitview_plugin_indexer::Indexer;
use bitview_primitives::{Index40, TxInIndex, TxOutIndex};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;
use tracing::{info, warn};
use vecdb::{AnyStoredVec, AnyVec, Error as VecError, ReadableVec, Stamp, VecIndex, WritableVec};

use super::{Vecs, bootstrap};
use bitview_compute::{undo_chunks, undo_tail_start};

impl Vecs {
    pub(crate) fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let starting_lengths = indexer.compute_start();

        let dep_version = indexer.vecs().inputs.txout_index.version()
            + indexer.vecs().outputs.first_txout_index.version()
            + indexer.vecs().inputs.first_txin_index.version()
            + indexer.vecs().outputs.value.version();
        {
            let _lock = exit.lock();
            if bootstrap::is_incomplete(self) {
                let spends = &mut self.spending_txin_index;
                indexer.rewrite_from(Height::ZERO, || Ok(spends.reset()?))?;
            }
            self.spending_txin_index
                .validate_computed_version_or_reset(dep_version)?;
        }
        let end = indexer.vecs().blocks.blockhash.len();
        if end == 0 {
            return Ok(());
        }

        let min_txout_index = {
            let _lock = exit.lock();
            // Zero means uninitialized; every checkpoint counts completed blocks.
            let starting_stamp = Stamp::from(starting_lengths.height.incremented());
            if self.spending_txin_index.stamp() >= starting_stamp {
                // Spends are rewritten in place: readers first fall back to the prefix the
                // rewrite keeps, where the rollback lands (a stamp counts completed blocks), or
                // genesis when it cannot land and the index rebuilds.
                let spends = &mut self.spending_txin_index;
                match spends.rollback_landing(starting_stamp)? {
                    Some(landing) => indexer.rewrite_from(Height::from(landing), || {
                        let landed = spends.rollback_before(starting_stamp)?;
                        debug_assert_eq!(landed, landing, "rollback lands where it planned");
                        Ok(())
                    })?,
                    None => {
                        warn!("Could not roll back spent outputs; rebuilding");
                        indexer.rewrite_from(Height::ZERO, || Ok(spends.reset()?))?;
                    }
                }
            }
            let min_txout_index = self
                .spending_txin_index
                .len()
                .min(starting_lengths.txout_index.to_usize());

            self.spending_txin_index
                .truncate_if_needed(TxOutIndex::from(min_txout_index))?;

            min_txout_index
        };

        // The block holding output `min_txout_index`: the last height whose first output is at
        // or below it (first_txout_index never decreases). Below the starting lengths it lies
        // under the starting height, whose first output (`starting_lengths.txout_index`) is past
        // it, so the search never reads that height, which may not be indexed.
        let first_txout_index_vec = &indexer.vecs().outputs.first_txout_index;
        let min_height = if min_txout_index == 0 {
            0
        } else if min_txout_index >= starting_lengths.txout_index.to_usize() {
            starting_lengths.height.to_usize()
        } else {
            let mut lo = 0usize;
            let mut hi = starting_lengths.height.to_usize();
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
            lo.saturating_sub(1)
        };
        debug_assert!(
            min_height <= starting_lengths.height.to_usize(),
            "txouts min_height ({min_height}) exceeds starting_lengths.height ({})",
            starting_lengths.height
        );

        // Only collect from min_height onward (not from 0)
        let first_txout_index_data = first_txout_index_vec.collect_range_at(min_height, end);
        let first_txin_index_data = indexer
            .vecs()
            .inputs
            .first_txin_index
            .collect_range_at(min_height, end);
        let first_txout = |height: usize| first_txout_index_data[height - min_height].to_usize();
        let first_txin = |height: usize| first_txin_index_data[height - min_height].to_usize();
        let txin_index_to_txout_index = &*indexer.vecs().inputs.txout_index;

        let mut start = min_height;
        let tail = undo_tail_start(&(start..end));
        if self.spending_txin_index.is_empty() && first_txout(tail) > bootstrap::RANGE_LEN {
            // Everything below the protected tail at once, its baseline unprotected like the
            // bulk's. An empty vector always starts at height zero.
            bootstrap::build(
                self,
                txin_index_to_txout_index,
                first_txin(tail),
                first_txout(tail),
                exit,
            )?;
            let _lock = exit.lock();
            self.spending_txin_index
                .stamped_write_maybe_with_changes(Stamp::from(Height::from(tail)), false)?;
            self.spending_txin_index.flush()?;
            start = tail;
        }
        for (blocks, with_changes) in undo_chunks(start..end) {
            // Fill the outputs up to the chunk's end.
            let txout_end = if blocks.end == end {
                indexer.vecs().outputs.value.len()
            } else {
                first_txout(blocks.end)
            };
            // Keep the chunk staged: fill_to may issue an unstamped write before
            // rollback data for a reorg truncation has been saved.
            let len = self.spending_txin_index.len();
            self.spending_txin_index.extend(repeat_n(
                Index40::new(TxInIndex::UNSPENT),
                txout_end.saturating_sub(len),
            ));

            let txin_start = first_txin(blocks.start);
            let txin_end = if blocks.end == end {
                txin_index_to_txout_index.len()
            } else {
                first_txin(blocks.end)
            };

            // Appended outputs are still in memory; only stored updates need sorting.
            let mut stored_updates = Vec::new();
            let stored_len = self.spending_txin_index.stored_len();
            let pushed = self.spending_txin_index.pushed_mut();
            let mut j = txin_start;
            txin_index_to_txout_index
                .try_for_each_range_at(txin_start, txin_end, |txout_index: TxOutIndex| {
                    if !txout_index.is_coinbase() {
                        let txin_index = Index40::new(TxInIndex::from(j));
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
                    len: self.spending_txin_index.len(),
                    name: self.spending_txin_index.name().to_owned(),
                })?;

            stored_updates.sort_unstable_by_key(|(txout_index, _)| *txout_index);
            let _lock = exit.lock();
            if with_changes {
                // Each stored output an appended block spends was unspent: its undo record
                // needs no read.
                self.spending_txin_index
                    .update_many_with_previous(stored_updates, Index40::new(TxInIndex::UNSPENT))?;
            } else {
                self.spending_txin_index.update_many(stored_updates)?;
            }
            // The bulk advances the baseline without undo; the protected tail saves its undo
            // before changing stored outputs.
            self.spending_txin_index.stamped_write_maybe_with_changes(
                Stamp::from(Height::from(blocks.end)),
                with_changes,
            )?;
            self.spending_txin_index.flush()?;
            if !with_changes {
                info!(
                    "Indexing spent outputs: {:.0}%",
                    blocks.end as f64 / end as f64 * 100.0
                );
            }
        }
        Ok(())
    }
}
