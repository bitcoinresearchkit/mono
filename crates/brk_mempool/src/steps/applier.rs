use std::sync::Arc;

use brk_types::Txid;

use crate::{
    Snapshot, TxRemoval,
    key::TxKey,
    state::State,
    steps::{
        Confirmed,
        preparer::{PoolDiff, TxAddition},
    },
    stores::Moment,
};

/// Applies a prepared diff to private working state: bury removals, insert additions, tell the buried what the new blocks confirmed, then
/// evict old tombstones. A burial keeps its chunk rate from the previous cycle's graph, or
/// its own fee rate if that graph lacks it.
pub fn apply(
    state: &mut State,
    prev_snapshot: &Snapshot,
    diff: PoolDiff,
    confirmed: &Confirmed,
    live: impl Fn(&Txid) -> bool,
) {
    let revision = state.graveyard.revision();
    let PoolDiff { added, removed } = diff;
    // A signaling one leaving lets go of what signaled through it alone.
    let mut orphaned = Vec::new();
    for (key, reason) in removed {
        if let Some(record) = state.txs.record_by_key(&key)
            && record.entry.rbf
        {
            orphaned.extend(state.pooled_children(&record.tx));
        }
        bury_one(state, prev_snapshot, &key, reason);
    }
    let mut entered = Vec::with_capacity(added.len());
    for TxAddition { tx, entry } in added {
        entered.push(entry.txid);
        state.insert(Arc::new(tx), entry);
    }
    state.inherit_signaling(&entered);
    state.recheck_signaling(orphaned);
    if !confirmed.is_empty() {
        state.graveyard.confirm(confirmed, &live, &mut state.upkeep);
    }
    state
        .graveyard
        .evict_old(Moment::now(), live, &mut state.upkeep);
    debug_assert!(
        revision == state.graveyard.revision() || state.graveyard.consistent(&state.upkeep),
        "graveyard invariants"
    );
}

fn bury_one(state: &mut State, prev_snapshot: &Snapshot, key: &TxKey, reason: TxRemoval) {
    let record = state.remove(key);
    let chunk_rate = prev_snapshot
        .chunk_rate_for(&record.entry.txid)
        .unwrap_or_else(|| record.own_rate());
    state
        .graveyard
        .bury(record, chunk_rate, reason, &mut state.upkeep);
}
