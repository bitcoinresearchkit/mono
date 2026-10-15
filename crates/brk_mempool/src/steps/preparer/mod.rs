//! Turn `Fetched` raws into a typed diff for the Applier. Pure CPU,
//! borrows the private `State` for the cycle. New entries are
//! classified into two buckets:
//!
//! - **revivable** - in the graveyard, resurrected from the tombstone.
//! - **fresh** - decoded from `new_raws`, prevouts resolved against
//!   the live mempool only. Confirmed-parent prevouts land as
//!   `prevout: None` and are filled post-apply by the resolver passed
//!   to `Mempool::tick_with`.
//!
//! Existing entries are not re-classified - they keep their first-sight
//! state on the live store. Removals are inferred by cross-referencing
//! inputs against the full `live_txids` set from the cycle's pull.

use bitcoin::Transaction as BitcoinTransaction;
use brk_types::{MempoolEntryInfo, Transaction, Txid, TxidPrefix, Vout};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    State,
    stores::{TxGraveyard, TxStore},
};

pub mod tx_addition;
pub mod tx_removal;
pub mod txs_pulled;

pub use tx_addition::TxAddition;
pub use tx_removal::TxRemoval;
pub use txs_pulled::TxsPulled;

pub fn prepare(
    live_txids: &[Txid],
    new_entries: Vec<MempoolEntryInfo>,
    new_txs: FxHashMap<Txid, BitcoinTransaction>,
    state: &State,
) -> TxsPulled {
    let live: FxHashSet<TxidPrefix> = live_txids.iter().map(TxidPrefix::from).collect();
    let added = classify_additions(new_entries, new_txs, &state.txs, &state.graveyard);
    let removed = classify_removals(&live, &added, &state.txs, &state.graveyard);

    TxsPulled {
        live_len: live.len(),
        added,
        removed,
    }
}

fn classify_additions(
    new_entries: Vec<MempoolEntryInfo>,
    mut new_txs: FxHashMap<Txid, BitcoinTransaction>,
    known: &TxStore,
    graveyard: &TxGraveyard,
) -> Vec<TxAddition> {
    new_entries
        .iter()
        .filter_map(|info| classify_addition(info, known, graveyard, &mut new_txs))
        .collect()
}

fn classify_addition(
    info: &MempoolEntryInfo,
    known: &TxStore,
    graveyard: &TxGraveyard,
    new_txs: &mut FxHashMap<Txid, BitcoinTransaction>,
) -> Option<TxAddition> {
    if known.contains(&info.txid) {
        return None;
    }
    if let Some(tomb) = graveyard.get(&info.txid) {
        return Some(TxAddition::revived(info, tomb));
    }
    let tx = new_txs.remove(&info.txid)?;
    Some(TxAddition::fresh(info, tx, known))
}

/// One `(prefix, reason)` per known tx that's gone from the live set,
/// in `known` iteration order.
///
/// Cost is `O(R * avg_inputs)` where R is the removed-tx count and
/// `avg_inputs` is small for non-pathological txs. Worst case is a
/// `mempoolminfee` jump dropping ~10k txs in one cycle - still well
/// under the cycle budget.
fn classify_removals(
    live: &FxHashSet<TxidPrefix>,
    added: &[TxAddition],
    known: &TxStore,
    graveyard: &TxGraveyard,
) -> Vec<(TxidPrefix, TxRemoval)> {
    let spent_by = build_spent_by(added, graveyard);
    known
        .records()
        .filter_map(|(prefix, record)| {
            if live.contains(prefix) {
                return None;
            }
            Some((*prefix, removal_reason(&record.tx, &spent_by)))
        })
        .collect()
}

fn removal_reason(tx: &Transaction, spent_by: &FxHashMap<(Txid, Vout), Txid>) -> TxRemoval {
    tx.input
        .iter()
        .find_map(|i| spent_by.get(&(i.txid, i.vout)).copied())
        .map_or(TxRemoval::Vanished, |by| TxRemoval::Replaced { by })
}

/// The outpoints this cycle's arrivals spend. A revived tx spends its inputs again too (it can
/// replace what entered while it was gone); its body is still in its tomb.
fn build_spent_by(added: &[TxAddition], graveyard: &TxGraveyard) -> FxHashMap<(Txid, Vout), Txid> {
    let mut spent_by: FxHashMap<(Txid, Vout), Txid> = FxHashMap::default();
    for addition in added {
        let tx = match addition {
            TxAddition::Fresh { tx, .. } => tx,
            TxAddition::Revived { entry } => match graveyard.get(&entry.txid) {
                Some(tomb) => tomb.tx.as_ref(),
                None => continue,
            },
        };
        for txin in &tx.input {
            spent_by.insert((txin.txid, txin.vout), tx.txid);
        }
    }
    spent_by
}
