use std::sync::Arc;

use brk_types::Transaction;

use crate::{
    Snapshot, TxRemoval,
    key::TxKey,
    state::{State, TxEntry},
    steps::preparer::{TxAddition, TxsPulled},
};

/// Applies a prepared diff to private working state: re-assert promoted
/// spends, bury removals, insert additions, then evict old tombstones.
/// `prev_snapshot` supplies the previous cycle's snapshot. Burial
/// reads each tomb's `chunk_rate` from it (always-fresh,
/// package-aware via local linearization). The fallback to
/// `entry.fee_rate()` is unreachable in steady state - every burial
/// target was alive at the previous tick, so the snapshot has it.
pub fn apply(state: &mut State, prev_snapshot: &Snapshot, pulled: TxsPulled) {
    let TxsPulled {
        live_len,
        added,
        promoted,
        removed,
    } = pulled;
    let additional = live_len.saturating_sub(state.txs.len());
    state.txs.reserve(additional);
    // Before burials, so a leaving listed spender never drops their mapping.
    for key in promoted {
        if let Some(record) = state.txs.record_by_key(&key) {
            state.outpoint_spends.insert_spends(&record.tx, key, true);
        }
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
    let Some(record) = state.txs.remove_by_key(key) else {
        return;
    };
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
