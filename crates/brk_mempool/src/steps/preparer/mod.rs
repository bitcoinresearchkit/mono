//! Turn `Fetched` raws into a typed diff for the Applier. Pure CPU,
//! borrows the private `State` for the cycle. Each new entry with a
//! fetched body becomes a [`TxAddition`], its prevouts resolved from the
//! pool as it stood before this cycle; the rest are filled after apply. A tx
//! back from the graveyard keeps its first sighting; the Applier exhumes its
//! tomb.
//!
//! Existing entries are not re-classified - they keep their first-sight
//! state on the live store. Removals are inferred by cross-referencing
//! inputs against the full `live_txids` set from the cycle's pull.

use bitcoin::Transaction as BitcoinTransaction;
use brk_types::{MempoolEntryInfo, Transaction, Txid, Vout};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{State, key::TxKey};

pub mod pool_diff;
pub mod tx_addition;
pub mod tx_removal;

pub use pool_diff::PoolDiff;
pub use tx_addition::TxAddition;
pub use tx_removal::TxRemoval;

pub fn prepare(
    live_txids: &[Txid],
    template_only: &FxHashSet<Txid>,
    new_entries: Vec<MempoolEntryInfo>,
    mut new_txs: FxHashMap<Txid, BitcoinTransaction>,
    state: &State,
) -> PoolDiff {
    let live: FxHashSet<TxKey> = live_txids.iter().map(TxKey::from).collect();
    let added: Vec<TxAddition> = new_entries
        .iter()
        .filter(|info| !state.txs.contains(&info.txid))
        .filter_map(|info| {
            let tx = new_txs.remove(&info.txid)?;
            let listed = !template_only.contains(&info.txid);
            Some(TxAddition::new(
                info,
                tx,
                listed,
                &state.txs,
                &state.graveyard,
            ))
        })
        .collect();
    let promoted = promoted(&live, template_only, state);
    let removed = classify_removals(&live, &added, &promoted, state);

    PoolDiff {
        live_len: live.len(),
        added,
        promoted,
        removed,
    }
}

/// Pooled txs template-only last poll and listed now: they take over their
/// outpoints like a listed arrival, so a listed spender leaving later never
/// drops their mapping.
fn promoted(live: &FxHashSet<TxKey>, template_only: &FxHashSet<Txid>, state: &State) -> Vec<TxKey> {
    state
        .template_only
        .iter()
        .filter(|txid| !template_only.contains(*txid))
        .map(TxKey::from)
        .filter(|key| live.contains(key) && state.txs.record_by_key(key).is_some())
        .collect()
}

/// One `(key, reason)` per pooled tx gone from the live set, in store order.
fn classify_removals(
    live: &FxHashSet<TxKey>,
    added: &[TxAddition],
    promoted: &[TxKey],
    state: &State,
) -> Vec<(TxKey, TxRemoval)> {
    let spent_by = build_spent_by(added, promoted, state);
    state
        .txs
        .records()
        .filter_map(|(key, record)| {
            if live.contains(key) {
                return None;
            }
            Some((
                *key,
                removal_reason(*key, &record.tx, &spent_by, live, state),
            ))
        })
        .collect()
}

/// Replaced when a conflicting tx arrives this cycle or is already live: a
/// stale template can keep the loser in the pool one or more cycles beside
/// its replacer, which is then no longer an arrival when the loser leaves.
/// The stored spender key is checked before any input scan, so a tx still
/// owning its own outpoints (mined, expired) costs one lookup per input.
fn removal_reason(
    key: TxKey,
    tx: &Transaction,
    spent_by: &FxHashMap<(Txid, Vout), Txid>,
    live: &FxHashSet<TxKey>,
    state: &State,
) -> TxRemoval {
    tx.input
        .iter()
        .find_map(|i| {
            spent_by.get(&(i.txid, i.vout)).copied().or_else(|| {
                state
                    .outpoint_spends
                    .spender_if(&state.txs, &i.txid, i.vout, |by| {
                        by != key && live.contains(&by)
                    })
                    .map(|(spender, _)| spender.entry.txid)
            })
        })
        .map_or(TxRemoval::Vanished, |by| TxRemoval::Replaced { by })
}

/// The outpoints this cycle's arrivals spend, revived txs included (one can
/// replace what entered while it was gone), then promoted ones: listed, they
/// win over a stale template arrival.
fn build_spent_by(
    added: &[TxAddition],
    promoted: &[TxKey],
    state: &State,
) -> FxHashMap<(Txid, Vout), Txid> {
    let mut spent_by: FxHashMap<(Txid, Vout), Txid> = FxHashMap::default();
    // `promoted` kept only pooled keys of this same state.
    let promoted = promoted.iter().map(|key| {
        state
            .txs
            .record_by_key(key)
            .expect("promoted tx is pooled")
            .tx
            .as_ref()
    });
    for tx in added.iter().map(|addition| &addition.tx).chain(promoted) {
        for txin in &tx.input {
            spent_by.insert((txin.txid, txin.vout), tx.txid);
        }
    }
    spent_by
}
