//! Working stores with frozen read data and shared immutable transaction bodies.

use brk_types::{TxOut, Vin};

use crate::key::TxKey;

pub mod addr_tracker;
pub mod cow;
pub mod live_payments;
pub mod outpoint_spends;
pub mod tx_graveyard;
pub mod tx_store;

pub use addr_tracker::AddrTracker;
pub use live_payments::LivePayments;
pub use outpoint_spends::OutpointSpends;
pub use tx_graveyard::{Moment, TxGraveyard, Upkeep};
pub use tx_store::{ReadOnlyTxStore, TxStore};

/// Folds newly resolved pooled inputs into addresses before changing their body and revision:
/// the one operation over both stores, so their individual fill mutators stay private here.
pub(crate) fn apply_fills(
    txs: &mut TxStore,
    addrs: &mut AddrTracker,
    key: &TxKey,
    fills: Vec<(Vin, TxOut)>,
) {
    let record = txs.record_by_key(key).expect("unresolved tx is pooled");
    addrs.resolve(record, fills.iter().map(|(_, prevout)| prevout));
    txs.apply_fills(key, fills);
}
