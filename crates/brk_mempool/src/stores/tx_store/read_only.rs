use brk_oracle::{HistogramRaw, Tally};
use brk_types::{MempoolTxSummary, Timestamp, Transaction, Txid};
use rapidhash::fast::RandomState;

use crate::{
    state::TxEntry,
    stores::{LivePayments, cow::CowIndexMap},
};

use super::TxRecord;

/// Query data shared by the working store and each completed publication.
#[derive(Clone, Default)]
pub struct ReadOnlyTxStore {
    pub(super) records: CowIndexMap<Txid, TxRecord, RandomState>,
    pub(super) content_revision: u64,
    pub(super) membership_revision: u64,
    /// The latest arrivals by Core's first-seen time, newest first.
    pub(super) recent: Vec<(Timestamp, MempoolTxSummary)>,
    pub(super) payments: LivePayments,
}

impl ReadOnlyTxStore {
    pub fn contains(&self, txid: &Txid) -> bool {
        self.records.contains_key(txid)
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
        self.records.get(txid)
    }

    /// `txid`'s position in [`Self::records`]'s order, if pooled.
    pub fn position(&self, txid: &Txid) -> Option<usize> {
        self.records.position(txid)
    }

    /// The record at `index` in dense storage order, [`Self::records`]'s.
    pub fn record_at(&self, index: usize) -> &TxRecord {
        self.records
            .get_index(index)
            .expect("an index within the store")
            .1
    }

    /// `(txid, record)` pairs in dense storage order.
    pub fn records(&self) -> impl Iterator<Item = (&Txid, &TxRecord)> {
        self.records.iter()
    }

    pub fn txids(&self) -> impl Iterator<Item = &Txid> {
        self.records.iter().map(|(txid, _)| txid)
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

    pub fn recent(&self) -> impl Iterator<Item = &MempoolTxSummary> {
        self.recent.iter().map(|(_, tx)| tx)
    }

    /// Snapshot the round-dollar-eligible histogram that feeds the oracle
    /// blend, with the live outputs' filter tally. Maintained incrementally, so
    /// this is `O(NUM_BINS)`, not `O(live_outputs)`.
    pub fn live_payments(&self) -> (HistogramRaw, Tally) {
        self.payments.payments()
    }
}
