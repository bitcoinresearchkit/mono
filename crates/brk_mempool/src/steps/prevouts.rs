//! A fresh tx can land in the store with `prevout: None` on some inputs: its
//! parent arrived in the same cycle, or is confirmed. [`fill`] runs after each
//! [`super::applier::apply`] and resolves both, confirmed parents through the
//! caller's resolver, then applies the fills and address changes. Published
//! transaction bodies remain immutable through `Arc::make_mut`; retries fill
//! only missing inputs.

#![allow(clippy::type_complexity)]

use brk_rpc::Client;
use brk_types::{TxOut, Txid, Vin, Vout};
use rapidhash::RapidHashMap;
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
    F: Fn(&[(Txid, Vout)]) -> RapidHashMap<(Txid, Vout), TxOut>,
{
    let (in_mempool, holes) = gather(&state.txs, &state.graveyard);
    let external = resolve_external(holes, resolver);
    let mut resolved = Vec::new();
    for (txid, fills) in in_mempool.into_iter().chain(external) {
        if state.txs.apply_fills(&TxKey::from(&txid), fills) {
            resolved.push(txid);
        }
    }
    state.fold_ready(resolved);
}

/// Default resolver: the node's UTXO set (`gettxout` per hole, batched), where a pending
/// transaction's confirmed parents' outputs still are: no `-txindex` needed.
pub fn rpc_resolver(
    client: Client,
) -> impl Fn(&[(Txid, Vout)]) -> RapidHashMap<(Txid, Vout), TxOut> {
    move |holes: &[(Txid, Vout)]| {
        if holes.is_empty() {
            return RapidHashMap::default();
        }
        client.get_tx_outs(holes).unwrap_or_else(|error| {
            warn!("mempool prevout fetch failed: {error}");
            RapidHashMap::default()
        })
    }
}

/// Single pass over `txs.unresolved()`: bucket each hole into a
/// same-cycle fill (the parent is live, or a retained vanished one) or an
/// external hole (parent is confirmed or unknown). Retained parents let the
/// children of a just-mined parent resolve before the chain publishes its block.
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
        let record = txs.record_by_key(key).expect("unresolved tx is pooled");
        let mut tx_fills: Vec<(Vin, TxOut)> = Vec::new();
        let mut tx_holes: Vec<(Vin, Txid, Vout)> = Vec::new();
        for (i, txin) in record.tx.input.iter().enumerate() {
            if txin.prevout.is_some() {
                continue;
            }
            let vin = Vin::from(i);
            if let Some(parent) = txs.get(&txin.txid).or_else(|| {
                graveyard
                    .get_unreplaced(&txin.txid)
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
/// consumer txs.
fn resolve_external<F>(
    holes: Vec<(Txid, Vec<(Vin, Txid, Vout)>)>,
    resolver: F,
) -> Vec<(Txid, Vec<(Vin, TxOut)>)>
where
    F: Fn(&[(Txid, Vout)]) -> RapidHashMap<(Txid, Vout), TxOut>,
{
    let total: usize = holes.iter().map(|(_, tx_holes)| tx_holes.len()).sum();
    let mut flat: Vec<(Txid, Vout)> = Vec::with_capacity(total);
    for (_, tx_holes) in &holes {
        for (_, prev_txid, vout) in tx_holes {
            flat.push((*prev_txid, *vout));
        }
    }
    let resolved = resolver(&flat);
    if resolved.is_empty() {
        return Vec::new();
    }
    holes
        .into_iter()
        .filter_map(|(txid, tx_holes)| {
            let fills: Vec<(Vin, TxOut)> = tx_holes
                .into_iter()
                .filter_map(|(vin, prev_txid, vout)| {
                    // Looked up, not taken: a template-only transaction and its listed replacement
                    // can both wait on one outpoint.
                    resolved
                        .get(&(prev_txid, vout))
                        .map(|output| (vin, output.clone()))
                })
                .collect();
            (!fills.is_empty()).then_some((txid, fills))
        })
        .collect()
}
