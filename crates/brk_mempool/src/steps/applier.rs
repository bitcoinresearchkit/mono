use std::sync::Arc;

use brk_types::Transaction;

use crate::{
    Snapshot, TxRemoval,
    key::TxKey,
    state::{State, TxEntry},
    steps::preparer::{PoolDiff, TxAddition},
};

/// Applies a prepared diff to private working state: re-assert promoted
/// spends, bury removals, insert additions, then evict old tombstones. A
/// burial keeps its chunk rate from the previous cycle's graph, or its own fee
/// rate if that graph lacks it.
pub fn apply(state: &mut State, prev_snapshot: &Snapshot, diff: PoolDiff) {
    let PoolDiff {
        live_len,
        added,
        promoted,
        removed,
    } = diff;
    let additional = live_len.saturating_sub(state.txs.len());
    state.txs.reserve(additional);
    // Before burials, so a leaving listed spender never drops their mapping.
    // `prepare` kept only pooled promoted keys and pooled removals, from this same state.
    for key in promoted {
        let record = state
            .txs
            .record_by_key(&key)
            .expect("promoted tx is pooled");
        state.outpoint_spends.insert_spends(&record.tx, key, true);
    }
    for (key, reason) in removed {
        bury_one(state, prev_snapshot, &key, reason);
    }
    for TxAddition { tx, entry, listed } in added {
        state.graveyard.exhume(&entry.txid);
        insert_one(state, tx, entry, listed);
    }
    state.graveyard.evict_old();
}

fn bury_one(state: &mut State, prev_snapshot: &Snapshot, key: &TxKey, reason: TxRemoval) {
    let record = state.txs.remove_by_key(key).expect("removed tx is pooled");
    let txid = record.entry.txid;
    let chunk_rate = prev_snapshot
        .chunk_rate_for(&txid)
        .unwrap_or_else(|| record.entry.fee_rate());
    state.info.remove(&record.tx, record.entry.fee);
    state.addrs.remove_tx(&record.tx);
    state.outpoint_spends.remove_spends(&record.tx, *key);
    state.graveyard.bury(record, chunk_rate, reason);
}

/// A listed tx owns its outpoints. A template-only one only claims free ones:
/// any other pooled spender is listed (the template is conflict-free), and a
/// stale template tx must not hide it, nor take its outpoints when leaving.
fn insert_one(state: &mut State, tx: Transaction, entry: TxEntry, listed: bool) {
    let tx = Arc::new(tx);
    let key = entry.key();
    state.info.add(&tx, entry.fee);
    state.addrs.add_tx(&tx);
    state.outpoint_spends.insert_spends(&tx, key, listed);
    state.txs.insert(tx, entry);
}
