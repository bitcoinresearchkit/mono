use std::{
    hash::{Hash, Hasher},
    ops::Deref,
    sync::Arc,
};

use brk_types::{MempoolRecentTx, Transaction, TxOut, Txid, Vin};
use rustc_hash::{FxHashSet, FxHasher};

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
            .filter_map(|(key, record)| {
                record
                    .tx
                    .input
                    .iter()
                    .any(|input| input.prevout.is_none())
                    .then_some(*key)
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

    pub fn insert(&mut self, tx: Arc<Transaction>, entry: TxEntry) {
        let key = entry.key();
        debug_assert!(
            !self.read_only.records.contains_key(&key),
            "TxKey collision: {key:?} already mapped"
        );
        let position = self.read_only.records.len();
        let txid = entry.txid;
        self.sample_recent(&entry.txid, &tx);
        if tx.input.iter().any(|i| i.prevout.is_none()) {
            self.unresolved.insert(key);
        }
        let record = TxRecord::new(tx, entry);
        self.read_only.histogram.add(&record);
        self.read_only.records.insert(key, record);
        self.read_only.txids_hash ^= Self::txid_position_hash(&txid, position);
        self.bump_content_revision();
    }

    fn sample_recent(&mut self, txid: &Txid, tx: &Transaction) {
        self.read_only
            .recent
            .insert(0, MempoolRecentTx::from((txid, tx)));
        self.read_only.recent.truncate(RECENT_CAP);
    }

    /// Remove by key and return the full record if present. `recent`
    /// is untouched: it's an "added" window, not a live-set mirror.
    pub fn remove_by_key(&mut self, key: &TxKey) -> Option<TxRecord> {
        let last_txid = self.read_only.records.last()?.1.entry.txid;
        let last_position = self.read_only.records.len() - 1;
        let (position, _, record) = self.read_only.records.swap_remove_full(key)?;
        self.read_only.txids_hash ^= Self::txid_position_hash(&record.entry.txid, position);
        if position != last_position {
            self.read_only.txids_hash ^= Self::txid_position_hash(&last_txid, last_position);
            self.read_only.txids_hash ^= Self::txid_position_hash(&last_txid, position);
        }
        self.unresolved.remove(key);
        self.read_only.histogram.remove(&record);
        self.bump_content_revision();
        Some(record)
    }

    /// Keys of the records with at least one unfilled prevout.
    pub fn unresolved(&self) -> &FxHashSet<TxKey> {
        &self.unresolved
    }

    /// Apply resolved prevouts to a tx in place. `fills` is `(vin, prevout)`, each for a
    /// missing prevout; `on_applied` sees every written prevout (the caller folds them into
    /// `AddrTracker`). Updates `unresolved` if fully resolved after the fill, and refreshes
    /// `total_sigop_cost` (P2SH and witness components depend on prevouts). `entry.vsize` is
    /// Core's value from `MempoolEntryInfo` and is not recomputed here - the sigops shift
    /// belongs to the `Transaction`, not the entry.
    pub fn apply_fills(
        &mut self,
        key: &TxKey,
        fills: Vec<(Vin, TxOut)>,
        mut on_applied: impl FnMut(&TxOut),
    ) {
        let record = self
            .read_only
            .records
            .get_mut(key)
            .expect("filled tx is pooled");
        let tx = Arc::make_mut(&mut record.tx);
        for (vin, prevout) in fills {
            let txin = &mut tx.input[usize::from(vin)];
            debug_assert!(txin.prevout.is_none(), "fills only fill missing prevouts");
            on_applied(txin.prevout.insert(prevout));
        }
        tx.refresh_sigops();
        record.digest = TxRecord::digest_of(tx);
        if record.tx.input.iter().all(|i| i.prevout.is_some()) {
            self.unresolved.remove(key);
        }
        self.bump_content_revision();
    }

    fn bump_content_revision(&mut self) {
        self.read_only.content_revision = self.read_only.content_revision.wrapping_add(1);
    }
}
