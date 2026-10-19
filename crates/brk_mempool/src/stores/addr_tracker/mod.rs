use std::{collections::hash_map::Entry as MapEntry, sync::Arc};

use brk_types::{AddressBytes, AddressMempoolStats, Transaction, TxOut, Txid};
use rustc_hash::FxHashMap;

pub mod addr_entry;

pub use addr_entry::AddrEntry;

/// Frozen maps share address records; only changed records copy their txid sets.
#[derive(Clone, Default)]
pub struct AddrTracker(FxHashMap<AddressBytes, Arc<AddrEntry>>);

impl AddrTracker {
    pub fn get(&self, addr: &AddressBytes) -> Option<&AddrEntry> {
        self.0.get(addr).map(Arc::as_ref)
    }

    pub fn add_tx(&mut self, tx: &Transaction) {
        let txid = &tx.txid;
        for txin in &tx.input {
            if let Some(prevout) = txin.prevout.as_ref() {
                self.add_input(txid, prevout);
            }
        }
        for txout in &tx.output {
            if let Some(bytes) = txout.addr_bytes() {
                self.apply_add(bytes, txid, |stats| stats.add_funding(txout));
            }
        }
    }

    pub fn remove_tx(&mut self, tx: &Transaction) {
        let txid = &tx.txid;
        for txin in &tx.input {
            if let Some(prevout) = txin.prevout.as_ref() {
                self.remove_input(txid, prevout);
            }
        }
        for txout in &tx.output {
            if let Some(bytes) = txout.addr_bytes() {
                self.apply_remove(bytes, txid, |stats| stats.remove_funding(txout));
            }
        }
    }

    /// Fold a resolved input into its address's stats; a prevout without an address is a no-op.
    pub fn add_input(&mut self, txid: &Txid, prevout: &TxOut) {
        let Some(bytes) = prevout.addr_bytes() else {
            return;
        };
        self.apply_add(bytes, txid, |stats| stats.add_spend(prevout));
    }

    fn remove_input(&mut self, txid: &Txid, prevout: &TxOut) {
        let Some(bytes) = prevout.addr_bytes() else {
            return;
        };
        self.apply_remove(bytes, txid, |stats| stats.remove_spend(prevout));
    }

    fn apply_add(
        &mut self,
        bytes: AddressBytes,
        txid: &Txid,
        update_stats: impl FnOnce(&mut AddressMempoolStats),
    ) {
        let entry = Arc::make_mut(self.0.entry(bytes).or_default());
        entry.txids.insert(*txid);
        update_stats(&mut entry.stats);
        entry.stats.tx_count = entry.txids.len() as u32;
    }

    fn apply_remove(
        &mut self,
        bytes: AddressBytes,
        txid: &Txid,
        update_stats: impl FnOnce(&mut AddressMempoolStats),
    ) {
        let MapEntry::Occupied(mut occupied) = self.0.entry(bytes) else {
            return;
        };
        let entry = Arc::make_mut(occupied.get_mut());
        entry.txids.remove(txid);
        update_stats(&mut entry.stats);
        let len = entry.txids.len();
        if len == 0 {
            occupied.remove_entry();
        } else {
            entry.stats.tx_count = len as u32;
        }
    }
}
