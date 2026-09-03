use std::collections::hash_map::Entry as MapEntry;

use brk_types::{AddrBytes, AddrMempoolStats, Transaction, TxOut, Txid};
use rustc_hash::FxHashMap;

use crate::cycle::AddrTransitions;

mod addr_entry;

pub use addr_entry::AddrEntry;

#[derive(Default)]
pub struct AddrTracker(FxHashMap<AddrBytes, AddrEntry>);

impl AddrTracker {
    pub fn get(&self, addr: &AddrBytes) -> Option<&AddrEntry> {
        self.0.get(addr)
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
                let entry = occupied.get_mut();
                entry.txids.insert(*txid);
                update_stats(&mut entry.stats);
                entry.stats.update_tx_count(entry.txids.len() as u32);
            }
            MapEntry::Vacant(vacant) => {
                let key = vacant.key().clone();
                let entry = vacant.insert(AddrEntry::default());
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
        let entry = occupied.get_mut();
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
    use brk_types::{Sats, SatsSigned, TxOut};

    use super::*;
    use crate::test_support::{fake_tx, p2wpkh_script};

    fn addr_of(script: &bitcoin::ScriptBuf) -> AddrBytes {
        AddrBytes::try_from(script).expect("p2wpkh script must yield AddrBytes")
    }

    #[test]
    fn add_tx_records_enter_for_new_addr() {
        let mut tracker = AddrTracker::default();
        let mut transitions = AddrTransitions::default();
        let out_script = p2wpkh_script(1);
        let tx = fake_tx(1, &[], &[(out_script.clone(), 5_000)]);
        let bytes = addr_of(&out_script);

        tracker.add_tx(&mut transitions, &tx);

        assert_eq!(tracker.len(), 1);
        let entry = tracker.get(&bytes).expect("addr indexed");
        assert_eq!(entry.stats.funded_txo_count, 1);
        assert_eq!(entry.stats.funded_txo_sum, Sats::from(5_000u64));
        assert_eq!(entry.stats.balance_delta, SatsSigned::from(5_000i64));
        assert_eq!(entry.stats.tx_count, 1);

        let (enters, leaves) = transitions.into_vecs();
        assert_eq!(enters, vec![bytes]);
        assert!(leaves.is_empty());
    }

    #[test]
    fn add_then_remove_tx_returns_to_zero_addrs() {
        let mut tracker = AddrTracker::default();
        let mut transitions = AddrTransitions::default();
        let out_script = p2wpkh_script(2);
        let prev_script = p2wpkh_script(3);
        let tx = fake_tx(
            2,
            &[Some(TxOut::from((
                prev_script.clone(),
                Sats::from(4_000u64),
            )))],
            &[(out_script.clone(), 3_500)],
        );
        let recv = addr_of(&out_script);
        let spend = addr_of(&prev_script);

        tracker.add_tx(&mut transitions, &tx);
        assert_eq!(
            tracker
                .get(&recv)
                .expect("receiving addr indexed")
                .stats
                .balance_delta,
            SatsSigned::from(3_500i64),
        );
        assert_eq!(
            tracker
                .get(&spend)
                .expect("spending addr indexed")
                .stats
                .balance_delta,
            SatsSigned::from(-4_000i64),
        );
        tracker.remove_tx(&mut transitions, &tx);
        assert_eq!(tracker.len(), 0);
        assert!(tracker.get(&recv).is_none());
        assert!(tracker.get(&spend).is_none());

        // add+remove in the same cycle: enter/leave cancel out.
        let (enters, leaves) = transitions.into_vecs();
        assert!(enters.is_empty(), "enter cancelled by same-cycle leave");
        assert!(leaves.is_empty(), "leave cancelled by same-cycle enter");
    }

    #[test]
    fn second_tx_touching_addr_does_not_re_enter() {
        let mut tracker = AddrTracker::default();
        let mut transitions = AddrTransitions::default();
        let shared = p2wpkh_script(4);
        let tx_a = fake_tx(3, &[], &[(shared.clone(), 2_500)]);
        let tx_b = fake_tx(4, &[], &[(shared.clone(), 7_500)]);

        tracker.add_tx(&mut transitions, &tx_a);
        tracker.add_tx(&mut transitions, &tx_b);

        let entry = tracker.get(&addr_of(&shared)).expect("addr indexed");
        assert_eq!(entry.stats.funded_txo_count, 2);
        assert_eq!(entry.stats.funded_txo_sum, Sats::from(10_000u64));
        assert_eq!(entry.stats.tx_count, 2);

        // Only one enter, even though two txs landed on the addr.
        let (enters, _) = transitions.into_vecs();
        assert_eq!(enters.len(), 1);
    }
}
