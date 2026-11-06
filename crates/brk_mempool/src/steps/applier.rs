use std::sync::Arc;

use brk_types::Transaction;

use crate::{
    Snapshot, TxRemoval,
    key::TxKey,
    state::{State, TxEntry},
    steps::{
        Confirmed,
        preparer::{PoolDiff, TxAddition},
    },
};

/// Applies a prepared diff to private working state: re-assert promoted
/// spends, bury removals, insert additions, tell the buried what the new blocks confirmed, then
/// evict old tombstones. A burial keeps its chunk rate from the previous cycle's graph, or
/// its own fee rate if that graph lacks it.
pub fn apply(state: &mut State, prev_snapshot: &Snapshot, diff: PoolDiff, confirmed: &Confirmed) {
    let PoolDiff {
        listed,
        added,
        promoted,
        removed,
    } = diff;
    // Before burials, so a leaving listed spender never drops their mapping.
    // `prepare` kept only pooled promoted keys and pooled removals, from this same state.
    let mut owners = Vec::with_capacity(promoted.len());
    for key in promoted {
        let record = state
            .txs
            .record_by_key(&key)
            .expect("promoted tx is pooled");
        owners.push(record.entry.txid);
        state.outpoint_spends.insert_spends(&record.tx, key, true);
    }
    // A waiting one leaving lets go of what it held back.
    let mut released = Vec::new();
    for (key, reason) in removed {
        if let Some(record) = state.txs.record_by_key(&key)
            && state.addrs.is_waiting(&record.entry.txid)
        {
            released.extend(state.pooled_children(&record.tx));
        }
        bury_one(state, prev_snapshot, &key, reason);
    }
    let mut entered = Vec::with_capacity(added.len());
    for TxAddition { tx, entry, listed } in added {
        state.graveyard.exhume(&entry.txid);
        entered.push(entry.txid);
        insert_one(state, tx, entry, listed);
    }
    // Every arrival waits until ready, and so does anything pooled it is a parent of (one coming
    // back to a child that stayed).
    for txid in &entered {
        state.hold_descendants(txid);
    }
    // A promoted one owns its outpoints again: what the cascades, which follow owners, missed.
    state.recheck(&owners);
    state.fold_ready(entered.iter().copied().chain(released).chain(owners));
    if !confirmed.is_empty() {
        state.graveyard.confirm(
            |txid| confirmed.mined(txid),
            |txid| confirmed.disconnected(txid),
            |tombstone| {
                let tx = &tombstone.record.tx;
                confirmed.conflict(&tx.txid, tx.input.iter().map(|i| (&i.txid, i.vout)))
            },
        );
    }
    // After the blocks, and only when every new one was read: a confirmed conflict outranks a
    // late arrival (with one unread, what vanished stays so).
    state.graveyard.index_vanished();
    if !confirmed.incomplete {
        for txid in &entered {
            if let Some(record) = state.txs.record(txid) {
                state.graveyard.relink(&record.tx);
            }
        }
    }
    // Pending is what Core lists, loaded or not: while the pool loads (after a restart, or past
    // the fetch cap) a replacer it hasn't fetched yet still keeps its tree.
    state
        .graveyard
        .evict_old(|txid| listed.contains(&TxKey::from(txid)));
}

fn bury_one(state: &mut State, prev_snapshot: &Snapshot, key: &TxKey, reason: TxRemoval) {
    let record = state.txs.remove_by_key(key).expect("removed tx is pooled");
    let txid = record.entry.txid;
    let chunk_rate = prev_snapshot
        .chunk_rate_for(&txid)
        .unwrap_or_else(|| record.own_rate());
    state.info.remove(record.entry.vsize, record.entry.fee);
    state.addrs.remove(&record);
    state.outpoint_spends.remove_spends(&record.tx, *key);
    state.graveyard.bury(record, chunk_rate, reason);
}

/// A listed tx owns its outpoints. A template-only one only claims free ones:
/// any other pooled spender is listed (the template is conflict-free), and a
/// stale template tx must not hide it, nor take its outpoints when leaving.
fn insert_one(state: &mut State, tx: Transaction, entry: TxEntry, listed: bool) {
    let tx = Arc::new(tx);
    let key = entry.key();
    state.info.add(entry.vsize, entry.fee);
    state.outpoint_spends.insert_spends(&tx, key, listed);
    state.txs.insert(tx, entry);
    let record = state.txs.record_by_key(&key).expect("just inserted");
    state.addrs.wait(record);
}
