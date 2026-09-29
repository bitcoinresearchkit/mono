use brk_error::{Error, OptionData, Result};

use std::ops::Range;

use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_indexer::{Indexer, Lengths};
use brk_oracle::{
    Config, Oracle, PaymentFilter, START_HEIGHT_FAST, START_HEIGHT_SLOW, bin_to_cents,
    cents_to_bin, pre_oracle_prices_from,
};
use brk_types::{Cents, OutputType, Sats, TxIndex, TxOutIndex, Weight};
use tracing::info;
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, StorageMode, VecIndex, WritableVec};

use super::Vecs;
use crate::Dependencies;

impl Vecs {
    fn compute_prices(&mut self, indexer: &Indexer) -> Result<()> {
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
            .validate_computed_version_or_reset(source_version)?;

        let total_heights = indexer.vecs().blocks.timestamp.len();

        // Reorg: truncate to starting_lengths
        self.spot
            .cents
            .height
            .truncate_if_needed_at(starting_height.to_usize())?;

        let seed_len = total_heights.min(START_HEIGHT_SLOW);
        let seed_start = self.spot.cents.height.len();
        for cents in pre_oracle_prices_from(seed_start).take(seed_len.saturating_sub(seed_start)) {
            self.spot.cents.height.push(cents);
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
            .data()?;
        let seed_bin = cents_to_bin(
            prev_cents
                .finite_inner()
                .ok_or(Error::Internal("Invalid oracle seed price"))? as f64,
        );
        let warmup = config.window_size.min(committed - START_HEIGHT_SLOW);
        let mut warmed = Ok(());
        let mut oracle = Oracle::from_checkpoint(seed_bin, config, |o| {
            warmed =
                Self::feed_blocks_for_warmup(o, indexer, (committed - warmup)..committed, None);
        });
        warmed?;

        let num_new = total_heights - committed;
        info!("Computing {num_new} oracle prices ({warmup} warmup blocks)...");

        // Slow cold-start EMA up to START_HEIGHT_FAST, then switch to the fast
        // mature-market EMA. Steady-state runs start past START_HEIGHT_FAST and skip
        // the slow segment entirely.
        {
            let mut processed = 0usize;
            let mut next_progress = 10u8;
            let mut push_ref_bin = |ref_bin| {
                self.spot
                    .cents
                    .height
                    .push(Cents::new(bin_to_cents(ref_bin)));

                processed += 1;
                let progress = (processed * 100 / num_new) as u8;
                while num_new >= 100 && progress >= next_progress {
                    info!("Oracle price computation: {next_progress}%");
                    next_progress += 10;
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
                )?;
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
                )?;
            }
        }

        info!("Computed {num_new} oracle prices.");

        Ok(())
    }

    /// Feed blocks into an Oracle when callers only need the warmed EMA/window state.
    pub fn feed_blocks_for_warmup<IM: StorageMode>(
        oracle: &mut Oracle,
        indexer: &Indexer<IM>,
        range: Range<usize>,
        cap: Option<&Lengths>,
    ) -> Result<()> {
        Self::feed_blocks_with(oracle, indexer, range, cap, |_, _, _| {})
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
    ) -> Result<()> {
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
        if range.start > range.end || range.end > height_len {
            return Err(Error::Internal("Invalid oracle block range"));
        }
        if range.is_empty() {
            return Ok(());
        }
        let collect_end = range.end.saturating_add(1).min(height_len);
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
        if first_tx_indexes.len() != collect_end - range.start
            || out_firsts.len() != collect_end - range.start
        {
            return Err(Error::Internal("Incomplete oracle block boundaries"));
        }

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
            let first_tx = first_tx_indexes[idx].to_usize();
            let block_first_tx = first_tx
                .checked_add(1)
                .ok_or(Error::Internal("Invalid oracle coinbase boundary"))?;
            let tx_count = next_first_tx_index
                .checked_sub(block_first_tx)
                .ok_or(Error::Internal("Invalid oracle transaction boundaries"))?;
            // Necessary serialization bounds, deliberately looser than full
            // consensus validation: at least 10 base bytes per transaction and
            // 9 per output, each costing four weight units.
            if next_first_tx_index > total_txs
                || tx_count >= u32::from(Weight::MAX_BLOCK) as usize / 40
            {
                return Err(Error::Internal(
                    "Oracle block transaction count exceeds bounds",
                ));
            }

            let out_end = out_firsts
                .get(idx + 1)
                .copied()
                .unwrap_or(TxOutIndex::from(total_outputs))
                .to_usize();

            let block_out_start = out_firsts[idx].to_usize();
            let block_outputs = out_end
                .checked_sub(block_out_start)
                .ok_or(Error::Internal("Invalid oracle block output boundaries"))?;
            if out_end > total_outputs || block_outputs > u32::from(Weight::MAX_BLOCK) as usize / 36
            {
                return Err(Error::Internal("Oracle block output count exceeds bounds"));
            }

            txout_cursor.advance(block_first_tx.checked_sub(txout_cursor.position()).ok_or(
                Error::Internal("Nonmonotonic oracle transaction boundaries"),
            )?);
            tx_starts.clear();
            txout_cursor.for_each(tx_count, |txout_index| {
                tx_starts.push(txout_index.to_usize());
            });
            if tx_starts.len() != tx_count
                || tx_starts
                    .iter()
                    .any(|start| *start < block_out_start || *start > out_end)
                || tx_starts.windows(2).any(|pair| pair[0] > pair[1])
            {
                return Err(Error::Internal(
                    "Invalid oracle transaction output boundaries",
                ));
            }
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
            if values.len() != out_end - out_start || output_types.len() != values.len() {
                return Err(Error::Internal("Incomplete oracle output data"));
            }

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
        Ok(())
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies { indexer } = dependencies;
        let exit = context.exit();

        self.db.sync_bg_tasks()?;

        self.compute_prices(indexer)?;
        {
            let _lock = exit.lock();
            self.spot.cents.height.write()?;
        }

        context.compact_database(&self.db);
        Ok(())
    }
}
