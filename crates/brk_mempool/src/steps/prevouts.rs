//! Prevout fill plumbing.
//!
//! A fresh tx can land in the store with `prevout: None` on some
//! inputs when the Preparer can't see the parent (parent arrived in
//! the same cycle as the child, or parent is confirmed and we don't
//! have an indexer hooked up). [`Prevouts::fill`] runs after each
//! successful `Applier::apply` and closes both gaps in one pass:
//!
//! 1. Snapshot under a read guard, walking `txs.unresolved()` once.
//!    For each hole, if the parent is also in the live pool we record
//!    a fill directly (cheap, lock-local). Otherwise we record the
//!    hole for external resolution.
//! 2. Drop the read guard. Call `resolver` on the remaining holes
//!    (typically `getrawtransaction` or an indexer lookup). Failures
//!    are simply skipped and retried next cycle.
//! 3. Take the write guard once and fold both fill batches into the
//!    `TxStore` via `apply_fills` -> `add_input`. Idempotent: each
//!    fill checks `prevout.is_none()` and bails if the tx was already
//!    removed or filled between phases.

use std::sync::atomic::{AtomicBool, Ordering};

use brk_rpc::Client;
use brk_types::{TxOut, Txid, TxidPrefix, Vin, Vout};
use parking_lot::RwLock;
use rustc_hash::{FxHashMap, FxHashSet};
use tracing::warn;

use crate::{cycle::CycleDiff, state::State, stores::TxStore};

pub struct Prevouts;

impl Prevouts {
    /// Fill every unfilled prevout the cycle can resolve. Same-cycle
    /// in-mempool parents are filled lock-locally. The remainder go
    /// through `resolver` (one batched call) outside any lock.
    pub fn fill<F>(lock: &RwLock<State>, diff: &mut CycleDiff, resolver: F)
    where
        F: Fn(&[(Txid, Vout)]) -> FxHashMap<(Txid, Vout), TxOut>,
    {
        let (in_mempool, holes) = {
            let state = lock.read();
            Self::gather(&state.txs)
        };
        let external = Self::resolve_external(holes, resolver);

        if in_mempool.is_empty() && external.is_empty() {
            return;
        }

        let mut state = lock.write();
        for (txid, fills) in in_mempool.into_iter().chain(external) {
            let prefix = TxidPrefix::from(&txid);
            for prevout in state.txs.apply_fills(&prefix, fills) {
                state.addrs.add_input(&mut diff.addrs, &txid, &prevout);
            }
        }
    }

    /// Default resolver: one batched `getrawtransaction` per cycle,
    /// deduped by parent txid. Requires bitcoind with `txindex=1`.
    pub fn rpc_resolver(
        client: Client,
    ) -> impl Fn(&[(Txid, Vout)]) -> FxHashMap<(Txid, Vout), TxOut> {
        let warned = AtomicBool::new(false);
        move |holes: &[(Txid, Vout)]| {
            if holes.is_empty() {
                return FxHashMap::default();
            }
            let mut seen: FxHashSet<Txid> = FxHashSet::default();
            let unique: Vec<Txid> = holes
                .iter()
                .filter_map(|(txid, _)| seen.insert(*txid).then_some(*txid))
                .collect();
            let parents = match client.get_raw_transactions(&unique) {
                Ok(map) => {
                    warned.store(false, Ordering::Relaxed);
                    map
                }
                Err(_) => {
                    if !warned.swap(true, Ordering::Relaxed) {
                        warn!(
                            "mempool: getrawtransaction batch failed; ensure bitcoind is running with txindex=1"
                        );
                    }
                    return FxHashMap::default();
                }
            };
            holes
                .iter()
                .filter_map(|(txid, vout)| {
                    let output = parents.get(txid)?.output.get(usize::from(*vout))?;
                    let txout = TxOut::from((output.script_pubkey.clone(), output.value.into()));
                    Some(((*txid, *vout), txout))
                })
                .collect()
        }
    }

    /// Single pass over `txs.unresolved()`: bucket each hole into a
    /// same-cycle in-mempool fill (parent is live) or an external hole
    /// (parent is confirmed or unknown).
    fn gather(
        txs: &TxStore,
    ) -> (
        Vec<(Txid, Vec<(Vin, TxOut)>)>,
        Vec<(Txid, Vec<(Vin, Txid, Vout)>)>,
    ) {
        let mut filled: Vec<(Txid, Vec<(Vin, TxOut)>)> = Vec::new();
        let mut holes: Vec<(Txid, Vec<(Vin, Txid, Vout)>)> = Vec::new();
        for prefix in txs.unresolved() {
            let Some(record) = txs.record_by_prefix(prefix) else {
                continue;
            };
            let mut tx_fills: Vec<(Vin, TxOut)> = Vec::new();
            let mut tx_holes: Vec<(Vin, Txid, Vout)> = Vec::new();
            for (i, txin) in record.tx.input.iter().enumerate() {
                if txin.prevout.is_some() {
                    continue;
                }
                let vin = Vin::from(i);
                if let Some(parent) = txs.get(&txin.txid)
                    && let Some(out) = parent.output.get(usize::from(txin.vout))
                {
                    tx_fills.push((vin, out.clone()));
                } else {
                    tx_holes.push((vin, txin.txid, txin.vout));
                }
            }
            let txid = record.entry.txid;
            if !tx_fills.is_empty() {
                filled.push((txid, tx_fills));
            }
            if !tx_holes.is_empty() {
                holes.push((txid, tx_holes));
            }
        }
        (filled, holes)
    }

    /// Flatten holes into one `(prev_txid, vout)` slice, invoke the
    /// resolver once, then re-attribute resolved entries to their
    /// consumer txs. Mempool double-spend rules guarantee every
    /// `(prev_txid, vout)` key is unique across the batch, so no
    /// dedup is needed before calling.
    fn resolve_external<F>(
        holes: Vec<(Txid, Vec<(Vin, Txid, Vout)>)>,
        resolver: F,
    ) -> Vec<(Txid, Vec<(Vin, TxOut)>)>
    where
        F: Fn(&[(Txid, Vout)]) -> FxHashMap<(Txid, Vout), TxOut>,
    {
        let total: usize = holes.iter().map(|(_, tx_holes)| tx_holes.len()).sum();
        let mut flat: Vec<(Txid, Vout)> = Vec::with_capacity(total);
        for (_, tx_holes) in &holes {
            for (_, prev_txid, vout) in tx_holes {
                flat.push((*prev_txid, *vout));
            }
        }
        let mut resolved = resolver(&flat);
        if resolved.is_empty() {
            return Vec::new();
        }
        holes
            .into_iter()
            .filter_map(|(txid, tx_holes)| {
                let fills: Vec<(Vin, TxOut)> = tx_holes
                    .into_iter()
                    .filter_map(|(vin, prev_txid, vout)| {
                        resolved
                            .remove(&(prev_txid, vout))
                            .map(|output| (vin, output))
                    })
                    .collect();
                (!fills.is_empty()).then_some((txid, fills))
            })
            .collect()
    }
}
