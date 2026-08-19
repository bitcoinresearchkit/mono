//! Verify the production restart property: an oracle restored via
//! `from_checkpoint` (seeded from the previous block's stored cents price,
//! replayed over the last `window_size` blocks) produces bit-exact `ref_bin`
//! values matching a continuously-running oracle from the restart height
//! onward.
//!
//! Mirrors the production transaction filter exactly, so it exercises the same
//! code path as the Bitview price plugin.
//!
//! Run with: cargo run -p brk_oracle --example determinism --release

use std::path::PathBuf;

use brk_oracle::{
    Config, HistogramRaw, Oracle, PaymentFilter, START_HEIGHT_FAST, START_HEIGHT_SLOW,
    bin_to_cents, cents_to_bin,
};
use brk_types::{OutputType, Sats, TxIndex, TxOutIndex};
use vecdb::{AnyVec, ReadableVec, VecIndex};

mod common;

struct Block {
    height: usize,
    values: Vec<Sats>,
    output_types: Vec<OutputType>,
    tx_starts: Vec<usize>,
    out_start: usize,
    out_end: usize,
}

fn build_histogram(block: &Block) -> HistogramRaw {
    let tx_outputs = (0..block.tx_starts.len()).map(|tx| {
        let lo = block.tx_starts[tx] - block.out_start;
        let hi = block
            .tx_starts
            .get(tx + 1)
            .map(|s| s - block.out_start)
            .unwrap_or(block.out_end - block.out_start);
        block.values[lo..hi]
            .iter()
            .copied()
            .zip(block.output_types[lo..hi].iter().copied())
    });
    PaymentFilter::for_height(block.height).histogram(tx_outputs)
}

fn main() {
    let data_dir = std::env::var("BRK_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap();
            PathBuf::from(home).join(".brk")
        });

    let indexer = common::import_indexer(&data_dir);
    let total_heights = indexer.vecs().blocks.timestamp.len();

    let fast_config = Config::default();
    let window_size = fast_config.window_size;

    let restart_offset = 1000;
    let end_offset = restart_offset + window_size * 4;
    let end_height = (START_HEIGHT_FAST + end_offset).min(total_heights);
    let restart_at = START_HEIGHT_FAST + restart_offset;
    let warmup_start = restart_at - window_size;
    let load_start = START_HEIGHT_SLOW;

    assert!(
        end_height > restart_at,
        "indexer has {total_heights} blocks; need at least {} to test restart at {restart_at}",
        restart_at + 1
    );

    println!(
        "Loading {} blocks ({load_start}..{end_height})...",
        end_height - load_start
    );
    let total_txs = indexer.vecs().transactions.txid.len();
    let total_outputs = indexer.vecs().outputs.value.len();
    let first_tx_index: Vec<TxIndex> = indexer.vecs().transactions.first_tx_index.collect();
    let out_first: Vec<TxOutIndex> = indexer.vecs().outputs.first_txout_index.collect();
    let mut txout_cursor = indexer
        .vecs()
        .transactions
        .first_txout_index
        .reader()
        .cursor();

    let mut blocks: Vec<Block> = Vec::with_capacity(end_height - load_start);
    for h in load_start..end_height {
        let ft = first_tx_index[h];
        let next_ft = first_tx_index
            .get(h + 1)
            .copied()
            .unwrap_or(TxIndex::from(total_txs));
        let block_first_tx = ft.to_usize() + 1;
        let tx_count = next_ft.to_usize() - block_first_tx;
        let out_end = out_first
            .get(h + 1)
            .copied()
            .unwrap_or(TxOutIndex::from(total_outputs))
            .to_usize();

        txout_cursor.advance(block_first_tx - txout_cursor.position());
        let mut tx_starts: Vec<usize> = Vec::with_capacity(tx_count);
        for _ in 0..tx_count {
            tx_starts.push(txout_cursor.next().unwrap().to_usize());
        }
        let out_start = tx_starts.first().copied().unwrap_or(out_end);

        let values: Vec<Sats> = indexer
            .vecs()
            .outputs
            .value
            .collect_range_at(out_start, out_end);
        let output_types: Vec<OutputType> = indexer
            .vecs()
            .outputs
            .output_type
            .collect_range_at(out_start, out_end);

        blocks.push(Block {
            height: h,
            values,
            output_types,
            tx_starts,
            out_start,
            out_end,
        });
    }

    let mut continuous = Oracle::from_seed();
    let continuous_bins: Vec<f64> = blocks
        .iter()
        .map(|b| {
            if b.height == START_HEIGHT_FAST {
                continuous.reconfigure(fast_config);
            }
            continuous.process_histogram(&build_histogram(b))
        })
        .collect();
    println!(
        "Continuous oracle: {} blocks processed",
        continuous_bins.len()
    );

    let prev_bin = continuous_bins[restart_at - load_start - 1];
    let seed_bin = cents_to_bin(bin_to_cents(prev_bin) as f64);
    println!(
        "Restart at {restart_at}: prev_bin={prev_bin:.4} -> cents -> seed_bin={seed_bin:.4} (delta {:.6})",
        seed_bin - prev_bin
    );

    let warmup_slice = &blocks[warmup_start - load_start..restart_at - load_start];
    let mut restored = Oracle::from_checkpoint(seed_bin, fast_config, |o| {
        for b in warmup_slice {
            o.process_histogram(&build_histogram(b));
        }
    });

    let restored_bins: Vec<f64> = blocks[restart_at - load_start..]
        .iter()
        .map(|b| restored.process_histogram(&build_histogram(b)))
        .collect();
    println!("Restored oracle: {} blocks processed", restored_bins.len());

    let mut mismatches: Vec<(usize, f64, f64)> = Vec::new();
    for (i, &r) in restored_bins.iter().enumerate() {
        let c = continuous_bins[restart_at - load_start + i];
        if r != c {
            mismatches.push((restart_at + i, c, r));
        }
    }

    println!();
    if mismatches.is_empty() {
        println!(
            "All {} blocks from {restart_at} onward match exactly.",
            restored_bins.len()
        );
    } else {
        println!(
            "{} of {} blocks differ (showing up to 5):",
            mismatches.len(),
            restored_bins.len()
        );
        for (h, c, r) in mismatches.iter().take(5) {
            println!(
                "  h={h}: continuous={c:.6}, restored={r:.6}, delta={:.6}",
                r - c
            );
        }
    }

    assert_eq!(
        mismatches.len(),
        0,
        "restored oracle diverged from continuous oracle"
    );

    println!();
    println!("Assertion passed: from_checkpoint restart is bit-exact.");
}
