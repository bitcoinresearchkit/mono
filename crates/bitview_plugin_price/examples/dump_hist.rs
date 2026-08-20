//! Dump the RAW per-output data over a height range for fully offline analysis.
//! Nothing is filtered or binned, so any downstream filter (round-BTC tolerance,
//! dust floor, type exclusion, OP_RETURN / batch-payout tx drops, log-bin
//! resolution) can be reconstructed in analysis WITHOUT re-dumping.
//!
//! For every non-coinbase output in [ORACLE_START, ORACLE_END) (default
//! 500000..510000) one row is written:
//!   oracle_outputs_{start}_{end}.csv   height,tx,sats,otype
//! where `tx` is the 0-based index of the (non-coinbase) transaction within the
//! block (so OP_RETURN-tx and >N-output-tx drops can be reapplied by grouping),
//! `sats` is the exact output value, and `otype` is `OutputType as u8`.
//!
//! Plus per-block metadata:
//!   oracle_meta_{start}_{end}.csv      height,timestamp,ex_low,ex_high,ex_close
//!
//! Run: cargo run -p bitview_plugin_price --example dump_hist --release

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use brk_types::{OutputType, Sats, TxIndex, TxOutIndex};
use vecdb::{AnyVec, ReadableVec, VecIndex};

mod common;

fn main() {
    let data_dir = std::env::var("BITVIEW_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap()).join(".bitview"));
    let out_dir = std::env::var("DUMP_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let start: usize = std::env::var("ORACLE_START")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(500_000);
    let end: usize = std::env::var("ORACLE_END")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(510_000);

    let indexer = common::import_indexer(&data_dir);
    let total_heights = indexer.vecs().blocks.timestamp.len();
    let end = end.min(total_heights);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let height_ohlc: Vec<[f64; 4]> = serde_json::from_str(
        &std::fs::read_to_string(format!("{manifest_dir}/examples/height_price_ohlc.json"))
            .expect("read height_price_ohlc.json"),
    )
    .expect("parse height OHLC");

    let timestamps: Vec<brk_types::Timestamp> = indexer.vecs().blocks.timestamp.collect();
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
    let mut tx_starts: Vec<usize> = Vec::new();

    let out_path = format!("{out_dir}/oracle_outputs_{start}_{end}.csv");
    let meta_path = format!("{out_dir}/oracle_meta_{start}_{end}.csv");
    let mut out_w = BufWriter::new(File::create(&out_path).expect("create outputs csv"));
    let mut meta_w = BufWriter::new(File::create(&meta_path).expect("create meta csv"));
    writeln!(out_w, "height,tx,sats,otype").unwrap();
    writeln!(meta_w, "height,timestamp,ex_low,ex_high,ex_close").unwrap();

    eprintln!(
        "otype legend: OpReturn={} P2TR={}",
        OutputType::OpReturn as u8,
        OutputType::P2TR as u8
    );

    let mut rows: u64 = 0;
    for h in start..end {
        let ft = first_tx_index[h];
        let next_ft = first_tx_index
            .get(h + 1)
            .copied()
            .unwrap_or(TxIndex::from(total_txs));
        let block_first_tx = ft.to_usize() + 1; // skip coinbase
        let tx_count = next_ft.to_usize() - block_first_tx;
        let out_end = out_first
            .get(h + 1)
            .copied()
            .unwrap_or(TxOutIndex::from(total_outputs))
            .to_usize();

        txout_cursor.advance(block_first_tx - txout_cursor.position());
        tx_starts.clear();
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

        for tx in 0..tx_count {
            let lo = tx_starts[tx] - out_start;
            let hi = tx_starts
                .get(tx + 1)
                .map(|s| s - out_start)
                .unwrap_or(out_end - out_start);
            for i in lo..hi {
                writeln!(out_w, "{h},{tx},{},{}", *values[i], output_types[i] as u8).unwrap();
                rows += 1;
            }
        }

        let o = height_ohlc.get(h).copied().unwrap_or([0.0; 4]);
        writeln!(
            meta_w,
            "{h},{},{:.2},{:.2},{:.2}",
            *timestamps[h], o[2], o[1], o[3]
        )
        .unwrap();
    }

    out_w.flush().unwrap();
    meta_w.flush().unwrap();
    eprintln!("wrote {out_path} ({rows} output rows)");
    eprintln!("wrote {meta_path}");
}
