use std::ops::Range;

use bitview_plugin_indexer::Indexer;
use bitview_primitives::Lengths;
use bitview_primitives::{Index40, TxOutIndex};
use brk_error::{Error, Result};
use brk_oracle::{Oracle, PaymentFilter};
use brk_types::{OutputType, Sats, TxIndex, Weight};
use rayon::prelude::*;
use vecdb::{AnyVec, ReadableVec, StorageMode, VecIndex};

/// Heights per batch: large range reads (memory-mapped small reads stall on page faults), and
/// histograms built in parallel.
const BATCH_HEIGHTS: usize = 1_000;

/// Feed blocks into an Oracle when callers only need the warmed EMA/window state.
pub fn feed_blocks_for_warmup<IM: StorageMode>(
    oracle: &mut Oracle,
    indexer: &Indexer<IM>,
    range: Range<usize>,
    cap: Option<&Lengths>,
) -> Result<()> {
    feed_blocks_with(oracle, indexer, range, cap, |_, _, _| {})
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

    let vecs = indexer.vecs();
    // Reused across batches: each batch's per-tx first outputs and output columns, read in large
    // range reads, then each block's histogram built in parallel and fed in block order.
    let mut tx_firsts: Vec<usize> = Vec::new();
    let mut values: Vec<Sats> = Vec::new();
    let mut output_types: Vec<OutputType> = Vec::new();
    let mut first_outputs: Vec<Index40<TxOutIndex>> = Vec::new();
    // Each block's non-coinbase transactions (absolute) and output end.
    let mut blocks: Vec<(Range<usize>, usize)> = Vec::new();

    for batch_start in (0..range.len()).step_by(BATCH_HEIGHTS) {
        let batch_end = (batch_start + BATCH_HEIGHTS).min(range.len());

        // Every block's bounds are checked before the batch is read: consecutive blocks share
        // their edges, so the batch's ranges are then ordered and bounded by the per-block limits.
        blocks.clear();
        for idx in batch_start..batch_end {
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
                || tx_count >= u64::from(Weight::MAX_BLOCK) as usize / 40
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
            if out_end > total_outputs || block_outputs > u64::from(Weight::MAX_BLOCK) as usize / 36
            {
                return Err(Error::Internal("Oracle block output count exceeds bounds"));
            }
            blocks.push((block_first_tx..next_first_tx_index, out_end));
        }

        let tx_base = first_tx_indexes[batch_start].to_usize();
        let tx_end = blocks.last().map_or(tx_base, |(txs, _)| txs.end);
        let out_base = out_firsts[batch_start].to_usize();
        let batch_out_end = blocks.last().map_or(out_base, |(_, out_end)| *out_end);
        vecs.transactions.first_txout_index.collect_range_into_at(
            tx_base,
            tx_end,
            &mut first_outputs,
        );
        tx_firsts.clear();
        tx_firsts.extend(first_outputs.iter().map(|first| first.get().to_usize()));
        vecs.outputs
            .value
            .collect_range_into_at(out_base, batch_out_end, &mut values);
        vecs.outputs
            .output_type
            .collect_range_into_at(out_base, batch_out_end, &mut output_types);
        if tx_firsts.len() != tx_end - tx_base
            || values.len() != batch_out_end - out_base
            || output_types.len() != values.len()
        {
            return Err(Error::Internal("Incomplete oracle output data"));
        }

        for (offset, (txs, out_end)) in blocks.iter_mut().enumerate() {
            let block_out_start = out_firsts[batch_start + offset].to_usize();
            *txs = txs.start - tx_base..txs.end - tx_base;
            let tx_starts = &tx_firsts[txs.clone()];
            if tx_starts
                .iter()
                .any(|start| *start < block_out_start || *start > *out_end)
                || tx_starts.windows(2).any(|pair| pair[0] > pair[1])
            {
                return Err(Error::Internal(
                    "Invalid oracle transaction output boundaries",
                ));
            }
        }

        let histograms = blocks
            .par_iter()
            .enumerate()
            .map(|(offset, (txs, out_end))| {
                let tx_starts = &tx_firsts[txs.clone()];
                let tx_outputs = (0..tx_starts.len()).map(|tx| {
                    let lo = tx_starts[tx] - out_base;
                    let hi = tx_starts.get(tx + 1).map_or(*out_end, |s| *s) - out_base;
                    values[lo..hi]
                        .iter()
                        .copied()
                        .zip(output_types[lo..hi].iter().copied())
                });
                PaymentFilter::for_height(range.start + batch_start + offset).histogram(tx_outputs)
            })
            .collect::<Vec<_>>();

        for (offset, hist) in histograms.iter().enumerate() {
            let ref_bin = oracle.process_histogram(hist);
            on_block(range.start + batch_start + offset, oracle, ref_bin);
        }
    }
    Ok(())
}
