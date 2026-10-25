use std::ops::Range;

use bitview_plugin_indexer::Indexer;
use bitview_primitives::{Index40, Lengths, TxOutIndex};
use brk_error::{Error, Result};
use brk_oracle::{HistogramRaw, Oracle, PaymentFilter, Tally};
use brk_types::{OutputType, Sats};
use rayon::prelude::*;
use vecdb::{AnyVec, ReadableVec, StorageMode, VecIndex};

/// Heights per batch: large range reads (memory-mapped small reads stall on page faults), and
/// blocks read in parallel.
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
    read_blocks(indexer, range, cap, &Histograms, |height, hist| {
        let ref_bin = oracle.process_histogram(&hist);
        on_block(height, oracle, ref_bin);
    })
}

/// The block at `height`'s outputs, each under the first rule of the oracle's filter that skips it,
/// or a payment.
pub fn block_tally<IM: StorageMode>(
    indexer: &Indexer<IM>,
    height: usize,
    cap: Option<&Lengths>,
) -> Result<Tally> {
    let mut tally = Tally::default();
    read_blocks(indexer, height..height + 1, cap, &Tallies, |_, t| tally = t)?;
    Ok(tally)
}

/// Each block's payments as the oracle filters them, at their exact amounts, in block order.
pub fn block_payments<IM: StorageMode>(
    indexer: &Indexer<IM>,
    range: Range<usize>,
    cap: Option<&Lengths>,
    each: impl FnMut(usize, Vec<Sats>),
) -> Result<()> {
    read_blocks(indexer, range, cap, &Payments, each)
}

/// What each block's outputs are read into, block by block in parallel: its coinbase's count, and
/// its other transactions' outputs.
trait ReadBlock: Sync {
    type Out: Send;

    fn read<Outputs>(
        &self,
        filter: PaymentFilter,
        coinbase: usize,
        txs: impl Iterator<Item = Outputs>,
    ) -> Self::Out
    where
        Outputs: ExactSizeIterator<Item = (Sats, OutputType)> + Clone;
}

/// The oracle's histogram of the block's payments.
struct Histograms;

impl ReadBlock for Histograms {
    type Out = HistogramRaw;

    fn read<Outputs>(
        &self,
        filter: PaymentFilter,
        _: usize,
        txs: impl Iterator<Item = Outputs>,
    ) -> HistogramRaw
    where
        Outputs: ExactSizeIterator<Item = (Sats, OutputType)> + Clone,
    {
        filter.histogram(txs)
    }
}

/// The block's payments, at their exact amounts.
struct Payments;

impl ReadBlock for Payments {
    type Out = Vec<Sats>;

    fn read<Outputs>(
        &self,
        filter: PaymentFilter,
        _: usize,
        txs: impl Iterator<Item = Outputs>,
    ) -> Vec<Sats>
    where
        Outputs: ExactSizeIterator<Item = (Sats, OutputType)> + Clone,
    {
        let mut payments = Vec::new();
        for outputs in txs {
            filter.for_each_payment(outputs, |sats| payments.push(sats));
        }
        payments
    }
}

/// Each output of the block, coinbase included, under the first rule that skips it, or a payment.
struct Tallies;

impl ReadBlock for Tallies {
    type Out = Tally;

    fn read<Outputs>(
        &self,
        filter: PaymentFilter,
        coinbase: usize,
        txs: impl Iterator<Item = Outputs>,
    ) -> Tally
    where
        Outputs: ExactSizeIterator<Item = (Sats, OutputType)> + Clone,
    {
        filter.tally(coinbase, txs)
    }
}

/// Read a range of blocks' outputs in batches (large range reads), each block read by `reader` in
/// parallel, then handed to `each` in block order.
fn read_blocks<IM: StorageMode, R: ReadBlock>(
    indexer: &Indexer<IM>,
    range: Range<usize>,
    cap: Option<&Lengths>,
    reader: &R,
    mut each: impl FnMut(usize, R::Out),
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
    // range reads, then each block read in parallel and handed over in block order.
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

        let blocks = (batch_start..batch_end)
            .into_par_iter()
            .map(|idx| {
                // Skip each block's coinbase: the oracle models payments, not issuance.
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
                // (The coinbase's outputs run up to the next transaction's, or the block's end.)
                let coinbase = txs.clone().next().map_or(block_out_end, first_output) - out_at(idx);
                reader.read(
                    PaymentFilter::for_height(range.start + idx),
                    coinbase,
                    tx_outputs,
                )
            })
            .collect::<Vec<_>>();

        for (offset, block) in blocks.into_iter().enumerate() {
            each(range.start + batch_start + offset, block);
        }
    }
    Ok(())
}
