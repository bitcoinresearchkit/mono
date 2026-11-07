use std::{collections::hash_map::Entry as MapEntry, sync::Arc};

use brk_types::{AddressBytes, AddressMempoolStats, Txid};
use rapidhash::fast::RandomState;
use rustc_hash::FxHashMap;

use crate::{
    key::TxKey,
    stores::{cow::CowMap, tx_store::TxRecord},
};

pub mod addr_entry;

pub use addr_entry::{AddrEntry, AddrTx};

/// Publications share the map's shards and its address records; a write copies only its shard
/// and the record it changes (with its txid set). A pooled
/// transaction folds in whole, once ready: every input resolved, and no pooled parent waiting
/// (what it spends of one is folded too). An address never sees half of one, nor a spend without
/// the pending output it spends. One still resolving (and what descends from it) shows up a cycle
/// later; meanwhile the addresses it pays (known from its outputs alone) answer `StateUpdating`
/// rather than leave it out.
#[derive(Clone, Default)]
pub struct AddrTracker {
    addrs: CowMap<AddressBytes, Arc<AddrEntry>, RandomState>,
    /// The pooled transactions not folded in yet, each with its inputs' count.
    waiting: CowMap<Txid, u32, RandomState>,
    /// The waiting transactions' keys folded together: it changes with the set.
    waiting_token: u64,
    /// The waiting transactions' inputs, all told: the most outpoints they can spend.
    waiting_inputs: usize,
    /// The waiting transactions paying each address, with how many of their outputs do: what it
    /// answers isn't complete yet, unless a newer block settled them all.
    awaited: CowMap<AddressBytes, FxHashMap<TxKey, u32>, RandomState>,
}

impl AddrTracker {
    pub fn get(&self, addr: &AddressBytes) -> Option<&AddrEntry> {
        self.addrs.get(addr).map(Arc::as_ref)
    }

    /// Whether the pooled `txid` waits to fold in (unresolved, or behind a parent that is).
    pub fn is_waiting(&self, txid: &Txid) -> bool {
        self.waiting.contains_key(txid)
    }

    /// Identifies the waiting transactions: what they spend isn't any address's yet (their
    /// prevouts aren't all known), only the outpoints'.
    pub fn waiting_token(&self) -> u64 {
        self.waiting_token
    }

    /// How many outpoints the waiting transactions spend, at most.
    pub fn waiting_inputs(&self) -> usize {
        self.waiting_inputs
    }

    /// The waiting transactions paying `addr`.
    pub fn awaited(&self, addr: &AddressBytes) -> impl Iterator<Item = &TxKey> {
        self.awaited.get(addr).into_iter().flat_map(|keys| keys.keys())
    }

    /// A transaction entering the pool (or held back again): it waits until ready.
    pub fn wait(&mut self, record: &TxRecord) {
        if self.waiting.contains_key(&record.entry.txid) {
            return;
        }
        let inputs = record.tx.input.len();
        self.waiting
            .insert(record.entry.txid, u32::try_from(inputs).expect("inputs fit 32 bits"));
        let key = record.entry.key();
        self.waiting_token ^= *key;
        self.waiting_inputs += inputs;
        for bytes in record.tx.output.iter().filter_map(|txout| txout.addr_bytes()) {
            *self
                .awaited
                .shard_mut(&bytes)
                .entry(bytes)
                .or_default()
                .entry(key)
                .or_default() += 1;
        }
    }

    /// No longer waiting: what it pays is complete again (as far as it goes).
    fn unwait(&mut self, record: &TxRecord) -> bool {
        let Some(inputs) = self.waiting.remove(&record.entry.txid) else {
            return false;
        };
        let key = record.entry.key();
        self.waiting_token ^= *key;
        self.waiting_inputs -= inputs as usize;
        for bytes in record.tx.output.iter().filter_map(|txout| txout.addr_bytes()) {
            let Some(keys) = self.awaited.get_mut(&bytes) else {
                continue;
            };
            if let MapEntry::Occupied(mut outputs) = keys.entry(key) {
                *outputs.get_mut() -= 1;
                if *outputs.get() == 0 {
                    outputs.remove();
                }
            }
            if keys.is_empty() {
                self.awaited.remove(&bytes);
            }
        }
        true
    }

    /// Folds a waiting transaction in, ready.
    pub fn fold(&mut self, record: &TxRecord) {
        let tx = &record.tx;
        debug_assert!(tx.is_resolved(), "only a resolved transaction folds in");
        let unwaited = self.unwait(record);
        debug_assert!(unwaited, "only a waiting transaction folds in");
        let at = AddrTx::from(record);
        for prevout in tx.input.iter().filter_map(|txin| txin.prevout.as_ref()) {
            if let Some(bytes) = prevout.addr_bytes() {
                self.apply_add(bytes, at, |stats| stats.add_spend(prevout));
            }
        }
        for txout in &tx.output {
            if let Some(bytes) = txout.addr_bytes() {
                self.apply_add(bytes, at, |stats| stats.add_funding(txout));
            }
        }
    }

    /// Folds a folded transaction back out: it waits again (a pooled parent of its does).
    pub fn unfold(&mut self, record: &TxRecord) {
        self.remove(record);
        self.wait(record);
    }

    /// A transaction leaving the pool, folded or waiting.
    pub fn remove(&mut self, record: &TxRecord) {
        if self.unwait(record) {
            return;
        }
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
