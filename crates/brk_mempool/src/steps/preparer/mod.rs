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

use super::Confirmed;
use crate::{State, stores::ReadOnlyTxStore};

pub mod pool_diff;
pub mod tx_addition;
pub mod tx_removal;

pub use pool_diff::PoolDiff;
pub use tx_addition::TxAddition;
pub use tx_removal::TxRemoval;

/// Core's listing set against the pool in one pass.
pub struct Listing {
    /// The listed txids the pool doesn't hold, in Core's listing order: asked in a random one,
    /// Core's lookups make a cold pool's first publication about 40 % slower.
    pub unpooled: Vec<Txid>,
    /// The pooled transactions the listing lacks, by position in [`ReadOnlyTxStore::records`]'s
    /// order.
    pub departed: Vec<usize>,
}

impl Listing {
    pub fn of(pool: &ReadOnlyTxStore, listing: Vec<Txid>) -> Self {
        let mut listed = vec![false; pool.len()];
        let mut unpooled = Vec::new();
        for txid in listing {
            match pool.position(&txid) {
                Some(position) => listed[position] = true,
                None => unpooled.push(txid),
            }
        }
        let departed = (0..listed.len())
            .filter(|&position| !listed[position])
            .collect();
        Self { unpooled, departed }
    }
}

pub fn prepare(
    departed: &[usize],
    new: impl IntoIterator<Item = (MempoolEntryInfo, BitcoinTransaction)>,
    confirmed: &Confirmed,
    state: &State,
) -> PoolDiff {
    let added: Vec<_> = new
        .into_iter()
        .map(|(info, tx)| TxAddition::new(&info, tx, &state.txs, &state.graveyard))
        .collect();
    PoolDiff {
        removed: classify_removals(departed, &added, confirmed, state),
        added,
    }
}

/// One `(txid, reason)` per pooled tx gone from the listing, in store order.
fn classify_removals(
    departed: &[usize],
    added: &[TxAddition],
    confirmed: &Confirmed,
    state: &State,
) -> Vec<(Txid, TxRemoval)> {
    let spent_by = build_spent_by(added);
    let mut removed: Vec<_> = departed
        .iter()
        .map(|&position| {
            let record = state.txs.record_at(position);
            (
                record.entry.txid,
                removal_reason(&record.tx, &spent_by, confirmed),
            )
        })
        .collect();
    TxRemoval::inherit(&mut removed, |txid| {
        state.txs.record(&txid).into_iter().flat_map(move |record| {
            state
                .outpoint_spends
                .children(&state.txs, &record.tx)
                .map(|child| child.entry.txid)
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
