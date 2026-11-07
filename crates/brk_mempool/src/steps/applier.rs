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

/// Applies a prepared diff to private working state: bury removals, insert additions, tell the buried what the new blocks confirmed, then
/// evict old tombstones. A burial keeps its chunk rate from the previous cycle's graph, or
/// its own fee rate if that graph lacks it.
pub fn apply(state: &mut State, prev_snapshot: &Snapshot, diff: PoolDiff, confirmed: &Confirmed) {
    let PoolDiff {
        listed,
        added,
        removed,
    } = diff;
    // A waiting one leaving lets go of what it held back; a signaling one, of what signaled
    // through it alone.
    let mut released = Vec::new();
    let mut orphaned = Vec::new();
    for (key, reason) in removed {
        if let Some(record) = state.txs.record_by_key(&key) {
            if state.addrs.is_waiting(&record.entry.txid) {
                released.extend(state.pooled_children(&record.tx));
            }
            if record.entry.rbf {
                orphaned.extend(state.pooled_children(&record.tx));
            }
        }
        bury_one(state, prev_snapshot, &key, reason);
    }
    let mut entered = Vec::with_capacity(added.len());
    for TxAddition { tx, entry } in added {
        state.graveyard.exhume(&entry.txid);
        entered.push(entry.txid);
        insert_one(state, tx, entry);
    }
    // Every arrival waits until ready, and so does anything pooled it is a parent of (one coming
    // back to a child that stayed).
    for txid in &entered {
        state.hold_descendants(txid);
    }
    state.fold_ready(entered.iter().copied().chain(released));
    state.inherit_signaling(&entered);
    state.recheck_signaling(orphaned);
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

fn insert_one(state: &mut State, tx: Transaction, entry: TxEntry) {
    let tx = Arc::new(tx);
    let key = entry.key();
    state.info.add(entry.vsize, entry.fee);
    state.outpoint_spends.insert_spends(&tx, key);
    state.txs.insert(tx, entry);
    let record = state.txs.record_by_key(&key).expect("just inserted");
    state.addrs.wait(record);
}
