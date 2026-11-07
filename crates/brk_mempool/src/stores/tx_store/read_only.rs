use brk_oracle::{HistogramRaw, Tally};
use brk_types::{MempoolRecentTx, Timestamp, Transaction, Txid};
use rustc_hash::FxBuildHasher;

use crate::{
    key::TxKey,
    state::TxEntry,
    stores::{LivePayments, cow::CowIndexMap},
};

use super::TxRecord;

/// Query data shared by the working store and each completed publication.
#[derive(Clone, Default)]
pub struct ReadOnlyTxStore {
    pub(super) records: CowIndexMap<TxKey, TxRecord, FxBuildHasher>,
    pub(super) content_revision: u64,
    pub(super) membership_revision: u64,
    /// The latest arrivals by Core's first-seen time, newest first.
    pub(super) recent: Vec<(Timestamp, MempoolRecentTx)>,
    pub(super) payments: LivePayments,
}

impl ReadOnlyTxStore {
    pub fn contains(&self, txid: &Txid) -> bool {
        self.get(txid).is_some()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn get(&self, txid: &Txid) -> Option<&Transaction> {
        self.record(txid).map(|record| record.tx.as_ref())
    }

    pub fn entry(&self, txid: &Txid) -> Option<&TxEntry> {
        self.record(txid).map(|record| &record.entry)
    }

    pub fn record(&self, txid: &Txid) -> Option<&TxRecord> {
        let record = self.records.get(&TxKey::from(txid))?;
        (record.entry.txid == *txid).then_some(record)
    }

    /// Lookup by key alone, without [`Self::record`]'s txid check: for keys taken from this
    /// store's own indexes.
    pub fn record_by_key(&self, key: &TxKey) -> Option<&TxRecord> {
        self.records.get(key)
    }

    /// The record at `index` in dense storage order, [`Self::records`]'s.
    pub fn record_at(&self, index: usize) -> &TxRecord {
        self.records
            .get_index(index)
            .expect("an index within the store")
            .1
    }

    /// `(key, record)` pairs in dense storage order.
    pub fn records(&self) -> impl Iterator<Item = (&TxKey, &TxRecord)> {
        self.records.iter()
    }

    pub fn txids(&self) -> impl Iterator<Item = &Txid> {
        self.records.values().map(|r| &r.entry.txid)
    }

    /// The revision of every mutation that can change the txid array or serialized tx bodies,
    /// unique in the process.
    pub fn content_revision(&self) -> u64 {
        self.content_revision
    }

    /// The revision of every insert and removal: of the txid array, order included (a removal
    /// moves the last record into its place).
    pub fn membership_revision(&self) -> u64 {
        self.membership_revision
    }

    pub fn recent(&self) -> impl Iterator<Item = &MempoolRecentTx> {
        self.recent.iter().map(|(_, tx)| tx)
    }

    /// Snapshot the round-dollar-eligible histogram that feeds the oracle
    /// blend, with the live outputs' filter tally. Maintained incrementally, so
    /// this is `O(NUM_BINS)`, not `O(live_outputs)`.
    pub fn live_payments(&self) -> (HistogramRaw, Tally) {
        self.payments.payments()
    }
}
