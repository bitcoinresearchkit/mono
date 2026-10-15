use std::sync::Arc;

use brk_types::{Transaction, TxidPrefix};

use crate::{
    Snapshot, TxRemoval,
    state::{State, TxEntry},
    steps::preparer::{TxAddition, TxsPulled},
};

/// Applies a prepared diff to private working state: bury removals, insert
/// additions, then evict old tombstones.
/// `prev_snapshot` supplies the previous cycle's snapshot. Burial
/// reads each tomb's `chunk_rate` from it (always-fresh,
/// package-aware via local linearization). The fallback to
/// `entry.fee_rate()` is unreachable in steady state - every burial
/// target was alive at the previous tick, so the snapshot has it.
pub fn apply(state: &mut State, prev_snapshot: &Snapshot, pulled: TxsPulled) {
    let TxsPulled {
        live_len,
        added,
        removed,
    } = pulled;
    let additional = live_len.saturating_sub(state.txs.len());
    state.txs.reserve(additional);
    for (prefix, reason) in removed {
        bury_one(state, prev_snapshot, &prefix, reason);
    }
    for addition in added {
        if let Some((tx, entry)) = resolve_addition(state, addition) {
            insert_one(state, tx, entry);
        }
    }
    state.graveyard.evict_old();
}

fn bury_one(state: &mut State, prev_snapshot: &Snapshot, prefix: &TxidPrefix, reason: TxRemoval) {
    let Some(record) = state.txs.remove_by_prefix(prefix) else {
        return;
    };
    let txid = record.entry.txid;
    let chunk_rate = prev_snapshot
        .chunk_rate_for(&txid)
        .unwrap_or_else(|| record.entry.fee_rate());
    state.info.remove(&record.tx, record.entry.fee);
    state.addrs.remove_tx(&record.tx);
    state.outpoint_spends.remove_spends(&record.tx, *prefix);
    state.graveyard.bury(record, chunk_rate, reason);
}

fn resolve_addition(
    state: &mut State,
    addition: TxAddition,
) -> Option<(Arc<Transaction>, TxEntry)> {
    match addition {
        TxAddition::Fresh { tx, entry } => Some((Arc::new(tx), entry)),
        TxAddition::Revived { entry } => {
            let tomb = state.graveyard.exhume(&entry.txid)?;
            Some((tomb.tx, entry))
        }
    }
}

fn insert_one(state: &mut State, tx: Arc<Transaction>, entry: TxEntry) {
    let prefix = entry.txid_prefix();
    state.info.add(&tx, entry.fee);
    state.addrs.add_tx(&tx);
    state.outpoint_spends.insert_spends(&tx, prefix);
    state.txs.insert(tx, entry);
}
