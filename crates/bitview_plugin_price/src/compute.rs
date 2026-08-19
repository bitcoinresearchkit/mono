use brk_error::Result;

use std::ops::Range;

use bitview_plugin::ComputePlugin;
use bitview_plugin_indexer::{Indexer, Lengths};
use brk_oracle::{
    Config, Oracle, PaymentFilter, START_HEIGHT_FAST, START_HEIGHT_SLOW, bin_to_cents, cents_to_bin,
};
use brk_types::{Cents, OutputType, Sats, TxIndex, TxOutIndex};
use tracing::info;
use vecdb::{AnyStoredVec, AnyVec, Exit, ReadableVec, StorageMode, VecIndex, WritableVec};

use super::Vecs;

impl Vecs {
    fn compute_inner(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        self.db.sync_bg_tasks()?;

        self.compute_prices(indexer, exit)?;

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });
        Ok(())
    }

    fn compute_prices(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        let source_version = [
            indexer.vecs().transactions.txid.version(),
            indexer.vecs().transactions.first_tx_index.version(),
            indexer.vecs().outputs.first_txout_index.version(),
            indexer.vecs().transactions.first_txout_index.version(),
            indexer.vecs().outputs.value.version(),
            indexer.vecs().outputs.output_type.version(),
        ]
        .into_iter()
        .sum();
        self.spot
            .cents
            .height
            .inner
            .validate_computed_version_or_reset(source_version)?;

        let total_heights = indexer.vecs().blocks.timestamp.len();

        if total_heights <= START_HEIGHT_SLOW {
            return Ok(());
        }

        // Reorg: truncate to starting_lengths
        let truncate_to = self.spot.cents.height.len().min(starting_height.to_usize());
        self.spot
            .cents
            .height
            .inner
            .truncate_if_needed_at(truncate_to)?;

        if self.spot.cents.height.len() < START_HEIGHT_SLOW {
            for cents in brk_oracle::pre_oracle_prices_from(self.spot.cents.height.len()) {
                if self.spot.cents.height.len() >= START_HEIGHT_SLOW {
                    break;
                }
                self.spot.cents.height.inner.push(cents);
            }
        }

        if self.spot.cents.height.len() >= total_heights {
            return Ok(());
        }

        let committed = self.spot.cents.height.len();
        let config = Config::for_height(committed);
        let prev_cents = self
            .spot
            .cents
            .height
            .collect_one_at(committed - 1)
            .unwrap();
        let seed_bin = cents_to_bin(prev_cents.inner() as f64);
        let warmup = config.window_size.min(committed - START_HEIGHT_SLOW);
        let mut oracle = Oracle::from_checkpoint(seed_bin, config, |o| {
            Self::feed_blocks_for_warmup(o, indexer, (committed - warmup)..committed, None);
        });

        let num_new = total_heights - committed;
        info!(
            "Computing oracle prices: {} to {} ({warmup} warmup)",
            committed, total_heights
        );

        // Slow cold-start EMA up to START_HEIGHT_FAST, then switch to the fast
        // mature-market EMA. Steady-state runs start past START_HEIGHT_FAST and skip
        // the slow segment entirely.
        {
            let mut processed = 0usize;
            let mut push_ref_bin = |ref_bin| {
                self.spot
                    .cents
                    .height
                    .inner
                    .push(Cents::new(bin_to_cents(ref_bin)));

                processed += 1;
                let progress = (processed * 100 / num_new) as u8;
                if processed > 1 && progress > (((processed - 1) * 100 / num_new) as u8) {
                    info!("Oracle price computation: {}%", progress);
                }
            };

            if committed < START_HEIGHT_FAST {
                let slow_end = START_HEIGHT_FAST.min(total_heights);
                Self::feed_blocks_with(
                    &mut oracle,
                    indexer,
                    committed..slow_end,
                    None,
                    |_, _, ref_bin| push_ref_bin(ref_bin),
                );
                if slow_end == START_HEIGHT_FAST {
                    oracle.reconfigure(Config::default());
                }
            }

            let fast_start = committed.max(START_HEIGHT_FAST);
            if fast_start < total_heights {
                Self::feed_blocks_with(
                    &mut oracle,
                    indexer,
                    fast_start..total_heights,
                    None,
                    |_, _, ref_bin| push_ref_bin(ref_bin),
                );
            }
        }

        {
            let _lock = exit.lock();
            self.spot.cents.height.inner.write()?;
        }

        info!(
            "Oracle prices complete: {} committed",
            self.spot.cents.height.len()
        );

        Ok(())
    }

    /// Feed blocks into an Oracle when callers only need the warmed EMA/window state.
    pub fn feed_blocks_for_warmup<IM: StorageMode>(
        oracle: &mut Oracle,
        indexer: &Indexer<IM>,
        range: Range<usize>,
        cap: Option<&Lengths>,
    ) {
        Self::feed_blocks_with(oracle, indexer, range, cap, |_, _, _| {});
    }

    /// Feed a range of blocks into an Oracle and call `on_block` after each
    /// processed block. This lets callers observe derived state such as EMA
    /// without duplicating the histogram extraction path.
    pub fn feed_blocks_with<IM: StorageMode>(
        oracle: &mut Oracle,
        indexer: &Indexer<IM>,
        range: Range<usize>,
        cap: Option<&Lengths>,
        mut on_block: impl FnMut(usize, &Oracle, f64),
    ) {
        let (total_txs, total_outputs, height_len) = match cap {
            Some(c) => (
                c.tx_index.to_usize(),
                c.txout_index.to_usize(),
                c.height.to_usize(),
            ),
            None => (
                indexer.vecs().transactions.txid.len(),
                indexer.vecs().outputs.value.len(),
                indexer.vecs().transactions.first_tx_index.len(),
            ),
        };

        // Pre-collect height-indexed data for the range (plus one extra for next-block lookups)
        let collect_end = (range.end + 1).min(height_len);
        let first_tx_indexes: Vec<TxIndex> = indexer
            .vecs()
            .transactions
            .first_tx_index
            .collect_range_at(range.start, collect_end);

        let out_firsts: Vec<TxOutIndex> = indexer
            .vecs()
            .outputs
            .first_txout_index
            .collect_range_at(range.start, collect_end);

        // Cursor avoids per-block PcoVec page decompression for the
        // tx-indexed first_txout_index lookup. Accessed tx_index values
        // are strictly increasing across blocks, so it only advances forward.
        let mut txout_cursor = indexer.vecs().transactions.first_txout_index.cursor();

        // Reusable buffers: avoid per-block allocation. `tx_starts` holds the
        // first txout index of each non-coinbase tx in the current block.
        let mut values: Vec<Sats> = Vec::new();
        let mut output_types: Vec<OutputType> = Vec::new();
        let mut tx_starts: Vec<usize> = Vec::new();

        for idx in 0..range.len() {
            let next_first_tx_index = first_tx_indexes
                .get(idx + 1)
                .copied()
                .unwrap_or(TxIndex::from(total_txs))
                .to_usize();
            let block_first_tx = first_tx_indexes[idx].to_usize() + 1;
            let tx_count = next_first_tx_index - block_first_tx;

            let out_end = out_firsts
                .get(idx + 1)
                .copied()
                .unwrap_or(TxOutIndex::from(total_outputs))
                .to_usize();

            txout_cursor.advance(block_first_tx - txout_cursor.position());
            tx_starts.clear();
            txout_cursor.for_each(tx_count, |txout_index| {
                tx_starts.push(txout_index.to_usize());
            });
            let out_start = tx_starts.first().copied().unwrap_or(out_end);

            indexer
                .vecs()
                .outputs
                .value
                .collect_range_into_at(out_start, out_end, &mut values);
            indexer.vecs().outputs.output_type.collect_range_into_at(
                out_start,
                out_end,
                &mut output_types,
            );

            let tx_outputs = (0..tx_count).map(|tx| {
                let lo = tx_starts[tx] - out_start;
                let hi = tx_starts
                    .get(tx + 1)
                    .map(|s| s - out_start)
                    .unwrap_or(out_end - out_start);
                values[lo..hi]
                    .iter()
                    .copied()
                    .zip(output_types[lo..hi].iter().copied())
            });
            let hist = PaymentFilter::for_height(range.start + idx).histogram(tx_outputs);

            let ref_bin = oracle.process_histogram(&hist);
            on_block(range.start + idx, oracle, ref_bin);
        }
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = crate::Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        exit: &Exit,
    ) -> Result<Self::Output> {
        self.compute_inner(dependencies.indexer, exit)
    }
}
