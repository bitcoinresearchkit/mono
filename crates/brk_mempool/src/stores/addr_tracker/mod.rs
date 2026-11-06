use std::{collections::hash_map::Entry as MapEntry, sync::Arc};

use brk_types::{AddressBytes, AddressMempoolStats, Transaction, Txid};
use rapidhash::RapidHashMap;
use rustc_hash::FxHashSet;

pub mod addr_entry;

pub use addr_entry::AddrEntry;

/// Frozen maps share address records; only changed records copy their txid sets. A pooled
/// transaction folds in whole, once ready: every input resolved, and no pooled parent waiting
/// (what it spends of one is folded too). An address never sees half of one, nor a spend without
/// the pending output it spends, so any publication answers for every address; one still
/// resolving (and what descends from it) shows up a cycle later.
#[derive(Clone, Default)]
pub struct AddrTracker {
    addrs: RapidHashMap<AddressBytes, Arc<AddrEntry>>,
    /// The pooled transactions not folded in yet.
    waiting: FxHashSet<Txid>,
}

impl AddrTracker {
    pub fn get(&self, addr: &AddressBytes) -> Option<&AddrEntry> {
        self.addrs.get(addr).map(Arc::as_ref)
    }

    /// Whether the pooled `txid` waits to fold in (unresolved, or behind a parent that is).
    pub fn is_waiting(&self, txid: &Txid) -> bool {
        self.waiting.contains(txid)
    }

    /// A transaction entering the pool: it waits until ready.
    pub fn wait(&mut self, txid: Txid) {
        self.waiting.insert(txid);
    }

    /// Folds a waiting transaction in, ready.
    pub fn fold(&mut self, tx: &Transaction) {
        debug_assert!(tx.is_resolved(), "only a resolved transaction folds in");
        let removed = self.waiting.remove(&tx.txid);
        debug_assert!(removed, "only a waiting transaction folds in");
        let txid = &tx.txid;
        for prevout in tx.input.iter().filter_map(|txin| txin.prevout.as_ref()) {
            if let Some(bytes) = prevout.addr_bytes() {
                self.apply_add(bytes, txid, |stats| stats.add_spend(prevout));
            }
        }
        for txout in &tx.output {
            if let Some(bytes) = txout.addr_bytes() {
                self.apply_add(bytes, txid, |stats| stats.add_funding(txout));
            }
        }
    }

    /// Folds a folded transaction back out: it waits again (a pooled parent of its does).
    pub fn unfold(&mut self, tx: &Transaction) {
        self.remove_tx(tx);
        self.waiting.insert(tx.txid);
    }

    /// A transaction leaving the pool, folded or waiting.
    pub fn remove_tx(&mut self, tx: &Transaction) {
        if self.waiting.remove(&tx.txid) {
            return;
        }
        let txid = &tx.txid;
        for prevout in tx.input.iter().filter_map(|txin| txin.prevout.as_ref()) {
            if let Some(bytes) = prevout.addr_bytes() {
                self.apply_remove(bytes, txid, |stats| stats.remove_spend(prevout));
            }
        }
        for txout in &tx.output {
            if let Some(bytes) = txout.addr_bytes() {
                self.apply_remove(bytes, txid, |stats| stats.remove_funding(txout));
            }
        }
    }

    fn apply_add(
        &mut self,
        bytes: AddressBytes,
        txid: &Txid,
        update_stats: impl FnOnce(&mut AddressMempoolStats),
    ) {
        let entry = Arc::make_mut(self.addrs.entry(bytes).or_default());
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
        let MapEntry::Occupied(mut occupied) = self.addrs.entry(bytes) else {
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
