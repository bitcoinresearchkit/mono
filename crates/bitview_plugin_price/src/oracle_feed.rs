use std::ops::Range;

use bitview_plugin_indexer::Indexer;
use bitview_primitives::Lengths;
use bitview_primitives::{Index40, TxOutIndex};
use brk_error::{Error, Result};
use brk_oracle::{Oracle, PaymentFilter};
use brk_types::{OutputType, Sats};
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
    let vecs = indexer.vecs();
    let (total_txs, total_outputs, height_len) = match cap {
        Some(c) => (
            c.tx_index.to_usize(),
            c.txout_index.to_usize(),
            c.height.to_usize(),
        ),
        None => (
            vecs.transactions.txid.len(),
            vecs.outputs.value.len(),
            vecs.transactions.first_tx_index.len(),
        ),
    };

    if range.start > range.end || range.end > height_len {
        return Err(Error::Internal("Invalid oracle block range"));
    }
    if range.is_empty() {
        return Ok(());
    }
    // One extra boundary for the last block's end.
    let collect_end = (range.end + 1).min(height_len);
    let first_tx_indexes = vecs
        .transactions
        .first_tx_index
        .collect_range_at(range.start, collect_end);
    let out_firsts = vecs
        .outputs
        .first_txout_index
        .collect_range_at(range.start, collect_end);
    let tx_at = |idx: usize| {
        first_tx_indexes
            .get(idx)
            .map_or(total_txs, |i| i.to_usize())
    };
    let out_at = |idx: usize| out_firsts.get(idx).map_or(total_outputs, |i| i.to_usize());

    // Reused across batches: each batch's per-tx first outputs and output columns, read in large
    // range reads, then each block's histogram built in parallel and fed in block order.
    let mut first_outputs: Vec<Index40<TxOutIndex>> = Vec::new();
    let mut values: Vec<Sats> = Vec::new();
    let mut output_types: Vec<OutputType> = Vec::new();

    for batch_start in (0..range.len()).step_by(BATCH_HEIGHTS) {
        let batch_end = (batch_start + BATCH_HEIGHTS).min(range.len());
        let (tx_base, tx_end) = (tx_at(batch_start), tx_at(batch_end));
        let (out_base, out_end) = (out_at(batch_start), out_at(batch_end));
        vecs.transactions.first_txout_index.collect_range_into_at(
            tx_base,
            tx_end,
            &mut first_outputs,
        );
        vecs.outputs
            .value
            .collect_range_into_at(out_base, out_end, &mut values);
        vecs.outputs
            .output_type
            .collect_range_into_at(out_base, out_end, &mut output_types);
        debug_assert!(
            first_outputs.len() == tx_end - tx_base
                && values.len() == out_end - out_base
                && output_types.len() == values.len(),
            "complete oracle output data"
        );
        let first_output = |tx: usize| first_outputs[tx - tx_base].get().to_usize();

        let histograms = (batch_start..batch_end)
            .into_par_iter()
            .map(|idx| {
                // The coinbase pays no one: skip each block's first transaction.
                let txs = tx_at(idx) + 1..tx_at(idx + 1);
                let block_out_end = out_at(idx + 1);
                debug_assert!(
                    txs.clone().all(|tx| first_output(tx) >= out_at(idx)
                        && first_output(tx) <= block_out_end
                        && (tx == txs.start || first_output(tx - 1) <= first_output(tx))),
                    "ordered transaction output boundaries inside their block"
                );
                let tx_outputs = txs.clone().map(|tx| {
                    let lo = first_output(tx) - out_base;
                    let hi = if tx + 1 < txs.end {
                        first_output(tx + 1)
                    } else {
                        block_out_end
                    } - out_base;
                    values[lo..hi]
                        .iter()
                        .copied()
                        .zip(output_types[lo..hi].iter().copied())
                });
                PaymentFilter::for_height(range.start + idx).histogram(tx_outputs)
            })
            .collect::<Vec<_>>();

        for (offset, hist) in histograms.iter().enumerate() {
            let ref_bin = oracle.process_histogram(hist);
            on_block(range.start + batch_start + offset, oracle, ref_bin);
        }
    }
    Ok(())
}
