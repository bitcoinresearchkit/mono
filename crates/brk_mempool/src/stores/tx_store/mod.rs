use std::{
    hash::{Hash, Hasher},
    ops::Deref,
    sync::Arc,
};

use brk_types::{MempoolRecentTx, Transaction, TxOut, Txid, TxidPrefix, Vin};
use rustc_hash::{FxHashSet, FxHasher};

use crate::state::TxEntry;

mod read_only;
pub mod tx_record;

pub use read_only::ReadOnlyTxStore;

pub use tx_record::TxRecord;

const RECENT_CAP: usize = 10;

/// Mutable transaction store. Only the writer owns the unresolved-input worklist.
#[derive(Default)]
pub struct TxStore {
    read_only: ReadOnlyTxStore,
    unresolved: FxHashSet<TxidPrefix>,
}

impl Deref for TxStore {
    type Target = ReadOnlyTxStore;

    fn deref(&self) -> &Self::Target {
        &self.read_only
    }
}

impl TxStore {
    pub fn from_read_only(read_only: ReadOnlyTxStore) -> Self {
        let unresolved = read_only
            .records()
            .filter_map(|(prefix, record)| {
                record
                    .tx
                    .input
                    .iter()
                    .any(|input| input.prevout.is_none())
                    .then_some(*prefix)
            })
            .collect();
        Self {
            read_only,
            unresolved,
        }
    }

    pub fn read_only_clone(&self) -> ReadOnlyTxStore {
        self.read_only.clone()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.read_only.records.reserve(additional);
    }

    fn txid_position_hash(txid: &Txid, position: usize) -> u64 {
        let mut hasher = FxHasher::default();
        (position as u64).hash(&mut hasher);
        txid.hash(&mut hasher);
        hasher.finish()
    }

    pub fn insert(&mut self, tx: impl Into<Arc<Transaction>>, entry: TxEntry) {
        let tx = tx.into();
        let prefix = entry.txid_prefix();
        debug_assert!(
            !self.read_only.records.contains_key(&prefix),
            "TxidPrefix collision: {prefix:?} already mapped. Birthday-rare on SHA-256d."
        );
        let position = self.read_only.records.len();
        let txid = entry.txid;
        self.sample_recent(&entry.txid, &tx);
        if tx.input.iter().any(|i| i.prevout.is_none()) {
            self.unresolved.insert(prefix);
        }
        let record = TxRecord { tx, entry };
        self.read_only.histograms.add(&record);
        self.read_only.records.insert(prefix, record);
        self.read_only.txids_hash ^= Self::txid_position_hash(&txid, position);
        self.bump_content_revision();
    }

    fn sample_recent(&mut self, txid: &Txid, tx: &Transaction) {
        self.read_only
            .recent
            .insert(0, MempoolRecentTx::from((txid, tx)));
        self.read_only.recent.truncate(RECENT_CAP);
    }

    /// Remove by prefix and return the full record if present. `recent`
    /// is untouched: it's an "added" window, not a live-set mirror.
    pub fn remove_by_prefix(&mut self, prefix: &TxidPrefix) -> Option<TxRecord> {
        let last_position = self.read_only.records.len().checked_sub(1)?;
        let last_txid = self.read_only.records.last()?.1.entry.txid;
        let (position, _, record) = self.read_only.records.swap_remove_full(prefix)?;
        self.read_only.txids_hash ^= Self::txid_position_hash(&record.entry.txid, position);
        if position != last_position {
            self.read_only.txids_hash ^= Self::txid_position_hash(&last_txid, last_position);
            self.read_only.txids_hash ^= Self::txid_position_hash(&last_txid, position);
        }
        self.unresolved.remove(prefix);
        self.read_only.histograms.remove(&record);
        self.bump_content_revision();
        Some(record)
    }

    /// Set of prefixes with at least one unfilled prevout. Used by the
    /// prevout filler as a cheap "is there any work?" gate.
    pub fn unresolved(&self) -> &FxHashSet<TxidPrefix> {
        &self.unresolved
    }

    /// Apply resolved prevouts to a tx in place. `fills` is `(vin, prevout)`.
    /// Returns the prevouts actually written (so the caller can fold them
    /// into `AddrTracker`). Updates `unresolved` if fully resolved after
    /// the fill, and refreshes `total_sigop_cost` (P2SH and witness
    /// components depend on prevouts). `entry.vsize` is Core's value from
    /// `MempoolEntryInfo` and is not recomputed here - the sigops shift
    /// belongs to the `Transaction`, not the entry.
    pub fn apply_fills(&mut self, prefix: &TxidPrefix, fills: Vec<(Vin, TxOut)>) -> Vec<TxOut> {
        let Some(record) = self.read_only.records.get_mut(prefix) else {
            return Vec::new();
        };
        if !fills.iter().any(|(vin, _)| {
            record
                .tx
                .input
                .get(usize::from(*vin))
                .is_some_and(|input| input.prevout.is_none())
        }) {
            return Vec::new();
        }
        let tx = Arc::make_mut(&mut record.tx);
        let applied = Self::write_prevouts(tx, fills);
        tx.refresh_sigops();
        if record.tx.input.iter().all(|i| i.prevout.is_some()) {
            self.unresolved.remove(prefix);
        }
        self.bump_content_revision();
        applied
    }

    fn bump_content_revision(&mut self) {
        self.read_only.content_revision = self.read_only.content_revision.wrapping_add(1);
    }

    fn write_prevouts(tx: &mut Transaction, fills: Vec<(Vin, TxOut)>) -> Vec<TxOut> {
        let mut applied = Vec::with_capacity(fills.len());
        for (vin, prevout) in fills {
            if let Some(txin) = tx.input.get_mut(usize::from(vin))
                && txin.prevout.is_none()
            {
                txin.prevout = Some(prevout.clone());
                applied.push(prevout);
            }
        }
        applied
    }
}
