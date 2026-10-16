//! Prevout fill plumbing.
//!
//! A fresh tx can land in the store with `prevout: None` on some
//! inputs when the Preparer can't see the parent (parent arrived in
//! the same cycle as the child, or parent is confirmed and we don't
//! have an indexer hooked up). [`fill`] runs after each
//! successful [`super::applier::apply`] and closes both gaps in one pass:
//!
//! Gather unresolved inputs from private state, resolve external parents, then
//! apply fills and address changes. Published transaction bodies remain immutable
//! through `Arc::make_mut`; retries fill only missing inputs.

use std::sync::atomic::{AtomicBool, Ordering};

use brk_rpc::Client;
use brk_types::{TxOut, Txid, Vin, Vout};
use rustc_hash::{FxHashMap, FxHashSet};
use tracing::warn;

use crate::{
    key::TxKey,
    state::State,
    stores::{TxGraveyard, TxStore},
};

/// Fill every unfilled prevout the cycle can resolve. Same-cycle
/// in-mempool parents are filled directly. The remainder go through
/// `resolver` in one batched call.
pub fn fill<F>(state: &mut State, resolver: F)
where
    F: Fn(&[(Txid, Vout)]) -> FxHashMap<(Txid, Vout), TxOut>,
{
    let (in_mempool, holes) = gather(&state.txs, &state.graveyard);
    let external = resolve_external(holes, resolver);

    if in_mempool.is_empty() && external.is_empty() {
        return;
    }

    for (txid, fills) in in_mempool.into_iter().chain(external) {
        state
            .txs
            .apply_fills(&TxKey::from(&txid), fills, |prevout| {
                state.addrs.add_input(&txid, prevout)
            });
    }
}

/// Default resolver: one batched `getrawtransaction` per cycle,
/// deduped by parent txid. Requires bitcoind with `txindex=1`.
pub fn rpc_resolver(client: Client) -> impl Fn(&[(Txid, Vout)]) -> FxHashMap<(Txid, Vout), TxOut> {
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
/// same-cycle fill (parent is live, or just left the pool: a live child's
/// vanished parent was mined) or an external hole (parent is confirmed or
/// unknown). Children of a just-mined parent resolve before the chain
/// publishes its block.
fn gather(
    txs: &TxStore,
    graveyard: &TxGraveyard,
) -> (
    Vec<(Txid, Vec<(Vin, TxOut)>)>,
    Vec<(Txid, Vec<(Vin, Txid, Vout)>)>,
) {
    let mut filled: Vec<(Txid, Vec<(Vin, TxOut)>)> = Vec::new();
    let mut holes: Vec<(Txid, Vec<(Vin, Txid, Vout)>)> = Vec::new();
    for key in txs.unresolved() {
        let Some(record) = txs.record_by_key(key) else {
            continue;
        };
        let mut tx_fills: Vec<(Vin, TxOut)> = Vec::new();
        let mut tx_holes: Vec<(Vin, Txid, Vout)> = Vec::new();
        for (i, txin) in record.tx.input.iter().enumerate() {
            if txin.prevout.is_some() {
                continue;
            }
            let vin = Vin::from(i);
            if let Some(parent) = txs.get(&txin.txid).or_else(|| {
                graveyard
                    .get_vanished(&txin.txid)
                    .map(|tomb| tomb.record.tx.as_ref())
            }) && let Some(out) = parent.output.get(usize::from(txin.vout))
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
