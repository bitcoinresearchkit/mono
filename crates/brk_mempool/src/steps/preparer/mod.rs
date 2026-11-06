//! Turn `Fetched` raws into a typed diff for the Applier. Pure CPU,
//! borrows the private `State` for the cycle. Each new entry with a
//! fetched body becomes a [`TxAddition`], its prevouts resolved from the
//! pool as it stood before this cycle; the rest are filled after apply. A tx
//! back from the graveyard keeps its first sighting; the Applier exhumes its
//! tomb.
//!
//! Existing entries are not re-classified - they keep their first-sight
//! state on the live store (a template's witness variant takes over a
//! record, its sighting kept). Removals are inferred by cross-referencing
//! inputs against the full `live_txids` set from the cycle's pull.

use bitcoin::Transaction as BitcoinTransaction;
use brk_types::{MempoolEntryInfo, Transaction, Txid, Vout};
use rapidhash::{RapidHashMap, RapidHashSet};
use rustc_hash::FxHashSet;

use super::Confirmed;
use crate::{State, key::TxKey};

pub mod pool_diff;
pub mod tx_addition;
pub mod tx_removal;

pub use pool_diff::PoolDiff;
pub use tx_addition::TxAddition;
pub use tx_removal::TxRemoval;

pub fn prepare(
    live_txids: &[Txid],
    template_only: &RapidHashSet<Txid>,
    new_entries: Vec<MempoolEntryInfo>,
    mut new_txs: RapidHashMap<Txid, BitcoinTransaction>,
    confirmed: &Confirmed,
    state: &State,
) -> PoolDiff {
    let live: FxHashSet<TxKey> = live_txids.iter().map(TxKey::from).collect();
    let added: Vec<TxAddition> = new_entries
        .iter()
        .filter_map(|info| {
            let tx = new_txs.remove(&info.txid)?;
            // The entry and the body are separate calls: a witness variant replacing the one
            // the entry describes in between leaves them unpaired (another weight). It stays
            // unloaded until a poll fetches both of one variant.
            (tx.compute_wtxid() == info.wtxid).then(|| {
                let listed = !template_only.contains(&info.txid);
                TxAddition::new(info, tx, listed, &state.txs, &state.graveyard)
            })
        })
        .collect();
    let promoted = promoted(&live, template_only, state);
    let mut removed = classify_removals(&live, &added, &promoted, confirmed, state);
    // A pooled txid fetched again is a template's witness variant: its record leaves for the
    // new body, which takes its place (and first sighting) in the same apply.
    removed.extend(
        added
            .iter()
            .map(|addition| addition.entry.key())
            .filter(|key| live.contains(key) && state.txs.record_by_key(key).is_some())
            .map(|key| (key, TxRemoval::Vanished)),
    );

    PoolDiff {
        added,
        promoted,
        removed,
    }
}

/// Pooled txs template-only last poll and listed now: they take over their
/// outpoints like a listed arrival, so a listed spender leaving later never
/// drops their mapping.
fn promoted(
    live: &FxHashSet<TxKey>,
    template_only: &RapidHashSet<Txid>,
    state: &State,
) -> Vec<TxKey> {
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
    confirmed: &Confirmed,
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
                removal_reason(*key, &record.tx, &spent_by, live, confirmed, state),
            ))
        })
        .collect()
}

/// Mined when a new block confirmed it. Replaced when a conflicting tx arrives this cycle, is
/// already live (a stale template can keep the loser in the pool one or more cycles beside its
/// replacer, which is then no longer an arrival when the loser leaves), or was confirmed by a new
/// block. The stored spender key is checked before any input scan, so a tx still owning its own
/// outpoints (expired) costs one lookup per input.
fn removal_reason(
    key: TxKey,
    tx: &Transaction,
    spent_by: &RapidHashMap<(Txid, Vout), Txid>,
    live: &FxHashSet<TxKey>,
    confirmed: &Confirmed,
    state: &State,
) -> TxRemoval {
    if confirmed.mined(&tx.txid) {
        return TxRemoval::Mined;
    }
    if let Some(by) = confirmed.conflict(&tx.txid, tx.input.iter().map(|i| (&i.txid, i.vout))) {
        return TxRemoval::Replaced { by };
    }
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
) -> RapidHashMap<(Txid, Vout), Txid> {
    let mut spent_by: RapidHashMap<(Txid, Vout), Txid> = RapidHashMap::default();
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
