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
use rapidhash::RapidHashMap;
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
            (tx.compute_wtxid() == info.wtxid)
                .then(|| TxAddition::new(info, tx, &state.txs, &state.graveyard))
        })
        .collect();
    let mut removed = classify_removals(&live, &added, confirmed, state);
    // A pooled txid fetched again is a witness variant (the template's, or one a restored record's
    // check found): its record leaves for the new body, which takes its place (and first sighting)
    // in the same apply.
    removed.extend(
        added
            .iter()
            .map(|addition| addition.entry.key())
            .filter(|key| live.contains(key) && state.txs.record_by_key(key).is_some())
            .map(|key| (key, TxRemoval::Vanished)),
    );

    PoolDiff {
        listed: live,
        added,
        removed,
    }
}

/// One `(key, reason)` per pooled tx gone from the live set, in store order.
fn classify_removals(
    live: &FxHashSet<TxKey>,
    added: &[TxAddition],
    confirmed: &Confirmed,
    state: &State,
) -> Vec<(TxKey, TxRemoval)> {
    let spent_by = build_spent_by(added);
    state
        .txs
        .records()
        .filter(|(key, _)| !live.contains(key))
        .map(|(key, record)| (*key, removal_reason(&record.tx, &spent_by, confirmed)))
        .collect()
}

/// Mined when a new block confirmed it. Replaced when a conflicting tx arrives this cycle or was
/// confirmed by a new block. The pool holds only what Core listed together last cycle, and Core's
/// pool is conflict-free: no other pooled tx can spend what a leaver spends.
fn removal_reason(
    tx: &Transaction,
    spent_by: &RapidHashMap<(Txid, Vout), Txid>,
    confirmed: &Confirmed,
) -> TxRemoval {
    if confirmed.mined(&tx.txid) {
        return TxRemoval::Mined;
    }
    if let Some(by) = confirmed
        .conflicts(&tx.txid, tx.input.iter().map(|i| (&i.txid, i.vout)))
        .next()
    {
        return TxRemoval::Replaced { by };
    }
    tx.input
        .iter()
        .find_map(|i| spent_by.get(&(i.txid, i.vout)).copied())
        .map_or(TxRemoval::Vanished, |by| TxRemoval::Replaced { by })
}

/// The outpoints this cycle's arrivals spend, revived txs included (one can
/// replace what entered while it was gone).
fn build_spent_by(added: &[TxAddition]) -> RapidHashMap<(Txid, Vout), Txid> {
    let mut spent_by: RapidHashMap<(Txid, Vout), Txid> = RapidHashMap::default();
    for tx in added.iter().map(|addition| &addition.tx) {
        for txin in &tx.input {
            spent_by.insert((txin.txid, txin.vout), tx.txid);
        }
    }
    spent_by
}
