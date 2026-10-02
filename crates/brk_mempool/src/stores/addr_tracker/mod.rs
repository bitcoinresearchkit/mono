use std::{collections::hash_map::Entry as MapEntry, sync::Arc};

use brk_types::{AddrBytes, AddrMempoolStats, Transaction, TxOut, Txid};
use rustc_hash::FxHashMap;

use crate::cycle::AddrTransitions;

pub mod addr_entry;

pub use addr_entry::AddrEntry;

/// Frozen maps share address records; only changed records copy their txid sets.
#[derive(Clone, Default)]
pub struct AddrTracker(FxHashMap<AddrBytes, Arc<AddrEntry>>);

impl AddrTracker {
    pub fn get(&self, addr: &AddrBytes) -> Option<&AddrEntry> {
        self.0.get(addr).map(Arc::as_ref)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn add_tx(&mut self, transitions: &mut AddrTransitions, tx: &Transaction) {
        let txid = &tx.txid;
        for txin in &tx.input {
            if let Some(prevout) = txin.prevout.as_ref() {
                self.add_input(transitions, txid, prevout);
            }
        }
        for txout in &tx.output {
            if let Some(bytes) = txout.addr_bytes() {
                self.apply_add(transitions, bytes, txid, |stats| stats.receiving(txout));
            }
        }
    }

    pub fn remove_tx(&mut self, transitions: &mut AddrTransitions, tx: &Transaction) {
        let txid = &tx.txid;
        for txin in &tx.input {
            if let Some(prevout) = txin.prevout.as_ref() {
                self.remove_input(transitions, txid, prevout);
            }
        }
        for txout in &tx.output {
            if let Some(bytes) = txout.addr_bytes() {
                self.apply_remove(transitions, bytes, txid, |stats| stats.received(txout));
            }
        }
    }

    /// Fold a single newly-resolved input into the per-address stats.
    /// Called by the prevout-fill paths after a prevout that was
    /// previously `None` has been filled, and by `add_tx` for each
    /// resolved input. Inputs whose prevout doesn't resolve to an addr
    /// are no-ops.
    pub fn add_input(&mut self, transitions: &mut AddrTransitions, txid: &Txid, prevout: &TxOut) {
        let Some(bytes) = prevout.addr_bytes() else {
            return;
        };
        self.apply_add(transitions, bytes, txid, |stats| stats.sending(prevout));
    }

    fn remove_input(&mut self, transitions: &mut AddrTransitions, txid: &Txid, prevout: &TxOut) {
        let Some(bytes) = prevout.addr_bytes() else {
            return;
        };
        self.apply_remove(transitions, bytes, txid, |stats| stats.sent(prevout));
    }

    fn apply_add(
        &mut self,
        transitions: &mut AddrTransitions,
        bytes: AddrBytes,
        txid: &Txid,
        update_stats: impl FnOnce(&mut AddrMempoolStats),
    ) {
        match self.0.entry(bytes) {
            MapEntry::Occupied(mut occupied) => {
                let entry = Arc::make_mut(occupied.get_mut());
                entry.txids.insert(*txid);
                update_stats(&mut entry.stats);
                entry.stats.update_tx_count(entry.txids.len() as u32);
            }
            MapEntry::Vacant(vacant) => {
                let key = vacant.key().clone();
                let entry = Arc::make_mut(vacant.insert(Arc::default()));
                entry.txids.insert(*txid);
                update_stats(&mut entry.stats);
                entry.stats.update_tx_count(entry.txids.len() as u32);
                transitions.record_enter(key);
            }
        }
    }

    fn apply_remove(
        &mut self,
        transitions: &mut AddrTransitions,
        bytes: AddrBytes,
        txid: &Txid,
        update_stats: impl FnOnce(&mut AddrMempoolStats),
    ) {
        let MapEntry::Occupied(mut occupied) = self.0.entry(bytes) else {
            return;
        };
        let entry = Arc::make_mut(occupied.get_mut());
        entry.txids.remove(txid);
        update_stats(&mut entry.stats);
        let len = entry.txids.len();
        if len == 0 {
            let (bytes, _) = occupied.remove_entry();
            transitions.record_leave(bytes);
        } else {
            entry.stats.update_tx_count(len as u32);
        }
    }
}

#[cfg(test)]
mod tests {
    use brk_types::Sats;

    use super::*;
    use crate::test_support::{fake_tx, p2wpkh_script};

    #[test]
    fn a_frozen_tracker_keeps_its_stats_and_transaction_membership() {
        let mut writer = AddrTracker::default();
        let mut transitions = AddrTransitions::default();
        let tx = fake_tx(1, &[], &[(p2wpkh_script(1), 1234)]);
        let address = tx.output[0].addr_bytes().unwrap();
        writer.add_tx(&mut transitions, &tx);
        let frozen = writer.clone();
        let second = fake_tx(2, &[], &[(p2wpkh_script(1), 2000)]);
        writer.add_tx(&mut transitions, &second);
        assert_eq!(writer.get(&address).unwrap().txids.len(), 2);
        assert_eq!(frozen.get(&address).unwrap().txids.len(), 1);
        assert_eq!(
            frozen.get(&address).unwrap().stats.funded_txo_sum,
            Sats::from(1234u64)
        );
        writer.remove_tx(&mut transitions, &tx);
        writer.remove_tx(&mut transitions, &second);
        assert!(writer.get(&address).is_none());
        assert!(frozen.get(&address).unwrap().txids.contains(&tx.txid));
    }
}
