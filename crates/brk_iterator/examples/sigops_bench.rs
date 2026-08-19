//! Microbenchmark: cost of `bitcoin::Transaction::total_sigop_cost` on
//! real recent blocks.
//!
//! Strategy: pull a sample of recent blocks via RPC (already-decoded
//! `bitcoin::Block`), and for each tx, time `total_sigop_cost` twice:
//!
//! 1. `|_| None` lookup — counts only legacy script_sig / script_pubkey
//!    sigops (skips P2SH redeem + witness). Cheap lower bound.
//! 2. Synthetic prevout map seeded with a P2WSH-shaped script_pubkey for
//!    every input, forcing the witness sigop walk to fire on every input.
//!    Pessimistic upper bound.
//!
//! The realistic cost is between these two, weighted by how many inputs
//! are actually P2SH/witness (~95% on mainnet today).
//!
//! Sample = N most recent blocks via `getblock` (verbosity 0 = raw bytes,
//! decoded by the iterator).

use std::time::Instant;

use bitcoin::{OutPoint, ScriptBuf, TxOut};
use brk_iterator::Blocks;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::Height;

fn main() -> brk_error::Result<()> {
    let bitcoin_dir = Client::default_bitcoin_path();
    let client = Client::new(
        Client::default_url(),
        Auth::CookieFile(bitcoin_dir.join(".cookie")),
    )?;
    let reader = Reader::new(bitcoin_dir.join("blocks"), &client);
    let blocks = Blocks::new(&client, &reader);

    let tip: u32 = client.get_block_count()? as u32;
    const SAMPLE_BLOCKS: u32 = 16;
    let start = Height::new(tip - SAMPLE_BLOCKS);
    let end = Height::new(tip);

    println!(
        "Sampling blocks {}..{} ({} blocks)",
        u32::from(start),
        u32::from(end),
        SAMPLE_BLOCKS
    );

    let mut all_txs: Vec<bitcoin::Transaction> = Vec::with_capacity(64_000);
    let mut total_inputs: usize = 0;
    let mut total_outputs: usize = 0;
    let mut total_witness_bytes: usize = 0;
    let mut total_script_sig_bytes: usize = 0;

    let t_fetch = Instant::now();
    for block in blocks.range(start, end)? {
        let block = block?;
        for tx in &block.txdata {
            total_inputs += tx.input.len();
            total_outputs += tx.output.len();
            for input in &tx.input {
                total_script_sig_bytes += input.script_sig.len();
                total_witness_bytes += input.witness.iter().map(|w| w.len()).sum::<usize>();
            }
            all_txs.push(tx.clone());
        }
    }
    let t_fetch = t_fetch.elapsed();

    let n = all_txs.len();
    println!(
        "Fetched {n} txs in {:?}: {} inputs, {} outputs, \
         scriptSig={} bytes, witness={} bytes",
        t_fetch, total_inputs, total_outputs, total_script_sig_bytes, total_witness_bytes
    );

    // 1) Cheap lower bound: |_| None lookup.
    let t1 = Instant::now();
    let mut sum_low: u64 = 0;
    for tx in &all_txs {
        sum_low += tx.total_sigop_cost(|_| None) as u64;
    }
    let elapsed_low = t1.elapsed();
    println!(
        "[None lookup ] {n} txs in {:?} = {:.0} ns/tx, sum sigops={}",
        elapsed_low,
        elapsed_low.as_nanos() as f64 / n as f64,
        sum_low
    );

    // 2) Pessimistic upper bound: P2WSH-shaped prevout for every input,
    // forcing the full witness walk. Use a 32-byte zero hash; the witness
    // last element will be empty so witness sigop count is 0, but the
    // is_p2wsh path runs end-to-end.
    let p2wsh_spk = {
        let mut bytes = vec![0x00, 0x20];
        bytes.extend_from_slice(&[0u8; 32]);
        ScriptBuf::from_bytes(bytes)
    };
    let synthetic_txout = TxOut {
        value: bitcoin::Amount::from_sat(0),
        script_pubkey: p2wsh_spk,
    };

    let t2 = Instant::now();
    let mut sum_hi: u64 = 0;
    for tx in &all_txs {
        sum_hi += tx.total_sigop_cost(|_op: &OutPoint| Some(synthetic_txout.clone())) as u64;
    }
    let elapsed_hi = t2.elapsed();
    println!(
        "[P2WSH lookup] {n} txs in {:?} = {:.0} ns/tx, sum sigops={}",
        elapsed_hi,
        elapsed_hi.as_nanos() as f64 / n as f64,
        sum_hi
    );

    // 3) Block-level extrapolation. Mainnet averages ~3000 tx/block, so
    // per-block cost ~= ns/tx * 3000.
    let txs_per_block = (n / SAMPLE_BLOCKS as usize) as f64;
    let block_low_us = elapsed_low.as_nanos() as f64 / SAMPLE_BLOCKS as f64 / 1000.0;
    let block_hi_us = elapsed_hi.as_nanos() as f64 / SAMPLE_BLOCKS as f64 / 1000.0;
    println!(
        "Per-block (avg {:.0} tx): low={:.1} us, high={:.1} us",
        txs_per_block, block_low_us, block_hi_us
    );

    Ok(())
}
