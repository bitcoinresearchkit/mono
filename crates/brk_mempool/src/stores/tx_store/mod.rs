use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use brk_types::{MempoolRecentTx, Transaction, TxOut, Txid, Vin};
use rustc_hash::FxHashSet;

use crate::{key::TxKey, state::TxEntry};

mod read_only;
pub mod tx_record;

pub use read_only::ReadOnlyTxStore;

pub use tx_record::TxRecord;

const RECENT_CAP: usize = 10;

/// Mutable transaction store. Only the writer owns the unresolved-input worklist.
#[derive(Default)]
pub struct TxStore {
    read_only: ReadOnlyTxStore,
    unresolved: FxHashSet<TxKey>,
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
            .filter_map(|(key, record)| (!record.tx.is_resolved()).then_some(*key))
            .collect();
        Self {
            read_only,
            unresolved,
        }
    }

    pub fn read_only_clone(&self) -> ReadOnlyTxStore {
        self.read_only.clone()
    }

    pub fn insert(&mut self, tx: Arc<Transaction>, entry: TxEntry) {
        let key = entry.key();
        debug_assert!(
            !self.read_only.records.contains_key(&key),
            "TxKey collision: {key:?} already mapped"
        );
        self.sample_recent(&entry, &tx);
        if !tx.is_resolved() {
            self.unresolved.insert(key);
        }
        let record = TxRecord::new(tx, entry);
        self.read_only.payments.add(&record);
        self.read_only.records.insert(key, record);
        self.bump_revisions(true);
    }

    /// Keeps the latest arrivals by first-seen time (a poll fetches in no particular order); among
    /// equal times, the later fetched first. A witness variant taking a record's place replaces
    /// its sample.
    fn sample_recent(&mut self, entry: &TxEntry, tx: &Transaction) {
        let recent = &mut self.read_only.recent;
        recent.retain(|(_, sample)| sample.txid != entry.txid);
        let at = recent.partition_point(|(seen, _)| *seen > entry.first_seen);
        if at < RECENT_CAP {
            recent.insert(
                at,
                (entry.first_seen, MempoolRecentTx::from((&entry.txid, tx))),
            );
            recent.truncate(RECENT_CAP);
        }
    }

    /// Remove by key and return the full record if present. `recent`
    /// is untouched: it's an "added" window, not a live-set mirror.
    pub fn remove_by_key(&mut self, key: &TxKey) -> Option<TxRecord> {
        let (_, _, record) = self.read_only.records.swap_remove_full(key)?;
        self.unresolved.remove(key);
        self.read_only.payments.remove(&record);
        self.bump_revisions(true);
        Some(record)
    }

    /// Keys of the records with at least one unfilled prevout.
    pub fn unresolved(&self) -> &FxHashSet<TxKey> {
        &self.unresolved
    }

    /// Apply resolved prevouts to a tx in place. `fills` is `(vin, prevout)`, each for a
    /// missing prevout. Updates `unresolved` if fully resolved after the fill, and refreshes
    /// `total_sigop_cost` (P2SH and witness components depend on prevouts). `entry.vsize` is
    /// Core's value from `MempoolEntryInfo` and is not recomputed here - the sigops shift
    /// belongs to the `Transaction`, not the entry.
    pub fn apply_fills(
        &mut self,
        key: &TxKey,
        fills: Vec<(Vin, TxOut)>,
    ) {
        let record = self
            .read_only
            .records
            .get_mut(key)
            .expect("filled tx is pooled");
        record.fill(fills);
        let resolved = record.tx.is_resolved();
        self.bump_revisions(false);
        if resolved {
            self.unresolved.remove(key);
        }
    }

    /// Whether `txid` signals replaceability (BIP125: itself, or through a pooled ancestor):
    /// whether that changed it.
    pub fn set_rbf(&mut self, txid: &Txid, rbf: bool) -> bool {
        let key = TxKey::from(txid);
        // Read first: a write copies the record's chunk while a publication shares it.
        if !self
            .read_only
            .records
            .get(&key)
            .is_some_and(|record| record.entry.txid == *txid && record.entry.rbf != rbf)
        {
            return false;
        }
        let record = self.read_only.records.get_mut(&key).expect("just read");
        record.entry.rbf = rbf;
        self.bump_revisions(false);
        true
    }

    /// The next of one count across the process: no two stores' contents share a revision. An
    /// insert or removal (`membership`) moves the txid array's too.
    fn bump_revisions(&mut self, membership: bool) {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let next = NEXT.fetch_add(1, Ordering::Relaxed);
        self.read_only.content_revision = next;
        if membership {
            self.read_only.membership_revision = next;
        }
    }
}
