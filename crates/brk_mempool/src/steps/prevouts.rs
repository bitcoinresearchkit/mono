//! A fresh tx can land in the store with `prevout: None` on some inputs: its
//! parent arrived in the same cycle, or is confirmed. [`fill`] runs after each
//! [`super::applier::apply`] and resolves both, confirmed parents through the
//! caller's resolver, then applies the fills and address changes. Published
//! transaction bodies remain immutable through `Arc::make_mut`; retries fill
//! only missing inputs.

#![allow(clippy::type_complexity)]

use brk_rpc::Client;
use brk_types::{Transaction, TxOut, Txid, Vin, Vout};
use rapidhash::{RapidHashMap, RapidHashSet};

use crate::{
    key::TxKey,
    state::State,
    stores::tx_store::TxRecord,
};

/// Where a fill goes: a pooled record, or block 0's template-only one at its index.
#[derive(Clone, Copy)]
enum FillTo {
    Pool(TxKey),
    TemplateOnly(usize),
}

/// Fill every unfilled prevout the cycle can resolve, the pool's and block 0's template-only
/// bodies' (served, and counted by their sigops). Same-cycle parents (pooled, template-only or
/// retained) are filled directly. The remainder go through `resolver` in one batched call, but
/// those whose parent is listed and not loaded yet (`unloaded`): the node's UTXO set can't have
/// them, they fill once it loads.
pub fn fill<F>(state: &mut State, resolver: F, unloaded: &RapidHashSet<Txid>)
where
    F: Fn(&[(Txid, Vout)]) -> RapidHashMap<(Txid, Vout), TxOut>,
{
    let template_only: RapidHashMap<Txid, usize> = state
        .template_only
        .iter()
        .enumerate()
        .map(|(i, record)| (record.entry.txid, i))
        .collect();
    let unresolved = state
        .txs
        .unresolved()
        .iter()
        .map(|key| {
            let record = state.txs.record_by_key(key).expect("unresolved tx is pooled");
            (FillTo::Pool(*key), record)
        })
        .chain(
            state
                .template_only
                .iter()
                .enumerate()
                .filter(|(_, record)| !record.tx.is_resolved())
                .map(|(i, record)| (FillTo::TemplateOnly(i), record)),
        );
    let parent = |txid: &Txid| {
        state
            .txs
            .get(txid)
            .or_else(|| template_only.get(txid).map(|&i| state.template_only[i].tx.as_ref()))
            .or_else(|| {
                state
                    .graveyard
                    .get_unreplaced(txid)
                    .map(|tomb| tomb.record.tx.as_ref())
            })
    };
    let (in_mempool, holes) = gather(unresolved, parent, unloaded);
    let external = resolve_external(holes, resolver);
    let mut resolved = Vec::new();
    for (into, fills) in in_mempool.into_iter().chain(external) {
        match into {
            FillTo::Pool(key) => {
                if state.txs.apply_fills(&key, fills) {
                    resolved.push(state.txs.record_by_key(&key).expect("filled").entry.txid);
                }
            }
            FillTo::TemplateOnly(i) => state.template_only[i].fill(fills),
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
        client.get_tx_outs(holes)
    }
}

/// Single pass over the `unresolved` records: bucket each hole into a same-cycle fill (`parent`
/// holds it: pooled, template-only, or a retained vanished one) or an external hole (parent is
/// confirmed or unknown); a hole whose parent is `unloaded` waits. Retained parents let the
/// children of a just-mined parent resolve before the chain publishes its block.
fn gather<'a, 'p>(
    unresolved: impl Iterator<Item = (FillTo, &'a TxRecord)>,
    parent: impl Fn(&Txid) -> Option<&'p Transaction>,
    unloaded: &RapidHashSet<Txid>,
) -> (
    Vec<(FillTo, Vec<(Vin, TxOut)>)>,
    Vec<(FillTo, Vec<(Vin, Txid, Vout)>)>,
) {
    let mut filled: Vec<(FillTo, Vec<(Vin, TxOut)>)> = Vec::new();
    let mut holes: Vec<(FillTo, Vec<(Vin, Txid, Vout)>)> = Vec::new();
    for (into, record) in unresolved {
        let mut tx_fills: Vec<(Vin, TxOut)> = Vec::new();
        let mut tx_holes: Vec<(Vin, Txid, Vout)> = Vec::new();
        for (i, txin) in record.tx.input.iter().enumerate() {
            if txin.prevout.is_some() {
                continue;
            }
            let vin = Vin::from(i);
            if let Some(out) =
                parent(&txin.txid).and_then(|parent| parent.output.get(usize::from(txin.vout)))
            {
                tx_fills.push((vin, out.clone()));
            } else if !unloaded.contains(&txin.txid) {
                tx_holes.push((vin, txin.txid, txin.vout));
            }
        }
        if !tx_fills.is_empty() {
            filled.push((into, tx_fills));
        }
        if !tx_holes.is_empty() {
            holes.push((into, tx_holes));
        }
    }
    (filled, holes)
}

/// Flatten holes into one `(prev_txid, vout)` slice, invoke the
/// resolver once, then re-attribute resolved entries to their
/// consumer txs.
fn resolve_external<F>(
    holes: Vec<(FillTo, Vec<(Vin, Txid, Vout)>)>,
    resolver: F,
) -> Vec<(FillTo, Vec<(Vin, TxOut)>)>
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
        .filter_map(|(into, tx_holes)| {
            let fills: Vec<(Vin, TxOut)> = tx_holes
                .into_iter()
                .filter_map(|(vin, prev_txid, vout)| {
                    resolved
                        .get(&(prev_txid, vout))
                        .map(|output| (vin, output.clone()))
                })
                .collect();
            (!fills.is_empty()).then_some((into, fills))
        })
        .collect()
}
