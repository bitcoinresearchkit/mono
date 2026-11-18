//! Turn fetched pairs into a typed diff for the Applier. Pure CPU,
//! borrows the private `State` for the cycle. Each new entry with a
//! fetched body becomes a [`TxAddition`], its prevouts resolved from the
//! pool as it stood before this cycle; the rest are filled after apply. A tx
//! back from the graveyard keeps its first sighting; the Applier exhumes its
//! tomb.
//!
//! Still-live entries keep their first sighting. Departures are classified from
//! confirmed conflicts and the inputs of newly admitted transactions.

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
    live: &FxHashSet<TxKey>,
    new: impl IntoIterator<Item = (MempoolEntryInfo, BitcoinTransaction)>,
    confirmed: &Confirmed,
    state: &State,
) -> PoolDiff {
    let added: Vec<_> = new
        .into_iter()
        .map(|(info, tx)| TxAddition::new(&info, tx, &state.txs, &state.graveyard))
        .collect();
    PoolDiff {
        removed: classify_removals(live, &added, confirmed, state),
        added,
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
    let mut removed: Vec<_> = state
        .txs
        .records()
        .filter(|(key, _)| !live.contains(key))
        .map(|(key, record)| (*key, removal_reason(&record.tx, &spent_by, confirmed)))
        .collect();
    TxRemoval::inherit(&mut removed, |key| {
        state
            .txs
            .record_by_key(&key)
            .into_iter()
            .flat_map(move |record| {
                state
                    .outpoint_spends
                    .children(&state.txs, key, &record.tx)
                    .map(|(key, _)| key)
            })
    });
    removed
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
        return TxRemoval::replaced(by);
    }
    tx.input
        .iter()
        .find_map(|i| spent_by.get(&(i.txid, i.vout)).copied())
        .map_or(TxRemoval::Vanished, TxRemoval::replaced)
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
