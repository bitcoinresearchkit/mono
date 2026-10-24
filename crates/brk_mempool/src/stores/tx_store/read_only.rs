use brk_oracle::HistogramRaw;
use brk_types::{MempoolRecentTx, Transaction, Txid};
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use crate::{key::TxKey, state::TxEntry, stores::LiveHistogram};

use super::TxRecord;

/// Query data shared by the working store and each completed publication.
#[derive(Clone, Default)]
pub struct ReadOnlyTxStore {
    pub(super) records: IndexMap<TxKey, TxRecord, FxBuildHasher>,
    pub(super) txids_hash: u64,
    pub(super) content_revision: u64,
    pub(super) recent: Vec<MempoolRecentTx>,
    pub(super) histogram: LiveHistogram,
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

    /// `(key, record)` pairs in dense storage order.
    pub fn records(&self) -> impl Iterator<Item = (&TxKey, &TxRecord)> {
        self.records.iter()
    }

    pub fn txids(&self) -> impl Iterator<Item = &Txid> {
        self.records.values().map(|r| &r.entry.txid)
    }

    pub fn txids_hash(&self) -> u64 {
        self.txids_hash
    }

    /// Process-local revision for every mutation that can change serialized tx bodies.
    pub fn content_revision(&self) -> u64 {
        self.content_revision
    }

    pub fn recent(&self) -> &[MempoolRecentTx] {
        &self.recent
    }

    /// Snapshot the round-dollar-eligible histogram that feeds the oracle
    /// blend. Maintained incrementally, so this is `O(NUM_BINS)`, not
    /// `O(live_outputs)`.
    pub fn live_eligible_histogram(&self) -> HistogramRaw {
        self.histogram.eligible()
    }
}
