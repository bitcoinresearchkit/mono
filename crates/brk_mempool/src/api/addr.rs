//! Address-keyed reads.

use std::cmp::Reverse;

use brk_types::{AddrBytes, AddrMempoolStats, Timestamp, Transaction, TxidPrefix};

use crate::Mempool;

impl Mempool {
    /// Per-address mempool stats. `None` if the address has no live mempool activity.
    pub fn addr_stats(&self, addr: &AddrBytes) -> Option<AddrMempoolStats> {
        self.read().addrs.get(addr).map(|e| e.stats.clone())
    }

    /// Process-local source revision and bounded transaction count for an address
    /// with live mempool activity. Used only to locate exact cached
    /// representations, never as an HTTP validator.
    pub fn addr_txs_source(&self, addr: &AddrBytes, limit: usize) -> Option<(u64, usize)> {
        let state = self.read();
        let entry = state.addrs.get(addr)?;
        Some((state.txs.content_revision(), entry.txids.len().min(limit)))
    }

    /// Live mempool txs touching `addr`, newest first by `first_seen`,
    /// capped at `limit`. Returns owned `Transaction`s.
    #[must_use]
    pub fn addr_txs(&self, addr: &AddrBytes, limit: usize) -> Vec<Transaction> {
        self.addr_txs_with_revision(addr, limit).0
    }

    /// Address transactions and the exact process-local source revision read
    /// under the same state guard. `None` means the address has no live activity.
    #[must_use]
    pub fn addr_txs_with_revision(
        &self,
        addr: &AddrBytes,
        limit: usize,
    ) -> (Vec<Transaction>, Option<u64>) {
        let state = self.read();
        let Some(entry) = state.addrs.get(addr) else {
            return (Vec::new(), None);
        };
        let revision = state.txs.content_revision();
        let mut ordered: Vec<(Timestamp, &Transaction)> = entry
            .txids
            .iter()
            .filter_map(|txid| {
                let record = state.txs.record_by_prefix(&TxidPrefix::from(txid))?;
                Some((record.entry.first_seen, &record.tx))
            })
            .collect();
        ordered.sort_unstable_by_key(|b| Reverse(b.0));
        let transactions = ordered
            .into_iter()
            .take(limit)
            .map(|(_, tx)| tx.clone())
            .collect();
        (transactions, Some(revision))
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{AddrBytes, Sats, TxOut};

    use super::*;
    use crate::{
        cycle::AddrTransitions,
        state::TxEntry,
        test_support::{fake_entry_info, fake_tx, p2wpkh_script},
    };

    #[test]
    fn source_and_transactions_share_revision_and_bounded_count() {
        let mempool = Mempool::for_test();
        let script = p2wpkh_script(1);
        let addr = AddrBytes::try_from(&script).unwrap();
        assert_eq!(mempool.addr_txs_source(&addr, 50), None);

        let mut transitions = AddrTransitions::default();
        let mut state = mempool.test_state_lock().write();
        for seed in 1..=2 {
            let prevout = TxOut::from((script.clone(), Sats::from(2_000u64)));
            let tx = fake_tx(seed, &[Some(prevout)], &[]);
            let entry = TxEntry::new(&fake_entry_info(tx.txid, 100, 100), 100, false);
            state.addrs.add_tx(&mut transitions, &tx);
            state.txs.insert(tx, entry);
        }
        drop(state);

        assert_eq!(mempool.addr_txs_source(&addr, 1), Some((2, 1)));
        let (transactions, revision) = mempool.addr_txs_with_revision(&addr, 1);
        assert_eq!(transactions.len(), 1);
        assert_eq!(revision, Some(2));
    }
}
