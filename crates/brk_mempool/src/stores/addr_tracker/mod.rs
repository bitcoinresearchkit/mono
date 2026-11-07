use std::{collections::hash_map::Entry as MapEntry, sync::Arc};

use brk_types::{AddressBytes, AddressMempoolStats, TxOut};
use rapidhash::fast::RandomState;

use crate::stores::{cow::CowMap, tx_store::TxRecord};

pub mod addr_entry;

pub use addr_entry::{AddrEntry, AddrTx};

/// Publications share the map's shards and its address records; a write copies only its shard
/// and the record it changes (with its txid set). A pooled transaction folds in as far as it is
/// known: its outputs on arrival, each input once its prevout is (what it spends of a pooled
/// parent only once that parent is pooled, so never a spend without the pending output it
/// spends). An address answers from what is known: what a still unresolved input spends shows up
/// once it resolves (its unspent outputs follow the outpoints, known before).
#[derive(Clone, Default)]
pub struct AddrTracker {
    addrs: CowMap<AddressBytes, Arc<AddrEntry>, RandomState>,
}

impl AddrTracker {
    pub fn get(&self, addr: &AddressBytes) -> Option<&AddrEntry> {
        self.addrs.get(addr).map(Arc::as_ref)
    }

    /// A transaction entering the pool: its outputs fold in, and its inputs already resolved.
    pub fn add(&mut self, record: &TxRecord) {
        let tx = &record.tx;
        self.resolve(record, tx.input.iter().filter_map(|txin| txin.prevout.as_ref()));
        let at = AddrTx::from(record);
        for txout in &tx.output {
            if let Some(bytes) = txout.addr_bytes() {
                self.apply_add(bytes, at, |stats| stats.add_funding(txout));
            }
        }
    }

    /// Prevouts of `record`'s inputs, resolved: what they spend folds in.
    pub fn resolve<'a>(&mut self, record: &TxRecord, prevouts: impl Iterator<Item = &'a TxOut>) {
        let at = AddrTx::from(record);
        for prevout in prevouts {
            if let Some(bytes) = prevout.addr_bytes() {
                self.apply_add(bytes, at, |stats| stats.add_spend(prevout));
            }
        }
    }

    /// A transaction leaving the pool: all of it that folded in folds back out.
    pub fn remove(&mut self, record: &TxRecord) {
        let tx = &record.tx;
        let at = AddrTx::from(record);
        for prevout in tx.input.iter().filter_map(|txin| txin.prevout.as_ref()) {
            if let Some(bytes) = prevout.addr_bytes() {
                self.apply_remove(bytes, &at, |stats| stats.remove_spend(prevout));
            }
        }
        for txout in &tx.output {
            if let Some(bytes) = txout.addr_bytes() {
                self.apply_remove(bytes, &at, |stats| stats.remove_funding(txout));
            }
        }
    }

    fn apply_add(
        &mut self,
        bytes: AddressBytes,
        tx: AddrTx,
        update_stats: impl FnOnce(&mut AddressMempoolStats),
    ) {
        let entry = Arc::make_mut(self.addrs.shard_mut(&bytes).entry(bytes).or_default());
        entry.txs.insert(tx);
        update_stats(&mut entry.stats);
        entry.stats.tx_count = entry.txs.len() as u32;
    }

    fn apply_remove(
        &mut self,
        bytes: AddressBytes,
        tx: &AddrTx,
        update_stats: impl FnOnce(&mut AddressMempoolStats),
    ) {
        if !self.addrs.contains_key(&bytes) {
            return;
        }
        let MapEntry::Occupied(mut occupied) = self.addrs.shard_mut(&bytes).entry(bytes) else {
            return;
        };
        let entry = Arc::make_mut(occupied.get_mut());
        entry.txs.remove(tx);
        update_stats(&mut entry.stats);
        let len = entry.txs.len();
        if len == 0 {
            occupied.remove_entry();
        } else {
            entry.stats.tx_count = len as u32;
        }
    }
}
