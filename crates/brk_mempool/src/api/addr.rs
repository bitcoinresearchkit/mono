//! Address-keyed reads.

use std::{cmp::Reverse, collections::BinaryHeap, sync::Arc};

use brk_error::Result;
use brk_types::{AddrBytes, AddrMempoolStats, BlockHash, Transaction, TxidPrefix};

use crate::ReadOnlyState;

impl ReadOnlyState {
    /// Statistics from a completed publication anchored to the requested chain.
    pub fn addr_stats(&self, addr: &AddrBytes, tip: &BlockHash) -> Result<AddrMempoolStats> {
        let state = self.pool()?;
        state.ensure_resolved_at(tip)?;
        Ok(state
            .addrs
            .get(addr)
            .map(|entry| entry.stats.clone())
            .unwrap_or_default())
    }

    /// Live mempool txs touching `addr`, newest first by `first_seen`,
    /// capped at `limit`. Shares immutable bodies from one completed publication;
    /// later prevout fills use copy-on-write and cannot mutate this selection.
    pub fn addr_txs(
        &self,
        addr: &AddrBytes,
        limit: usize,
        tip: &BlockHash,
    ) -> Result<Vec<Arc<Transaction>>> {
        let state = self.pool()?;
        state.ensure_resolved_at(tip)?;
        let Some(entry) = state.addrs.get(addr) else {
            return Ok(Vec::new());
        };
        if limit == 0 {
            return Ok(Vec::new());
        }
        // Retain only the requested page. The smallest timestamp is at the
        // heap root; prefix breaks equal-time ties deterministically.
        let mut newest = BinaryHeap::with_capacity(limit.min(entry.txids.len()));
        for txid in &entry.txids {
            let Some(record) = state.txs.record(txid) else {
                continue;
            };
            let candidate = Reverse((record.entry.first_seen, TxidPrefix::from(txid)));
            if newest.len() < limit {
                newest.push(candidate);
            } else if let Some(mut oldest) = newest.peek_mut()
                && candidate < *oldest
            {
                *oldest = candidate;
            }
        }
        let transactions = newest
            .into_sorted_vec()
            .into_iter()
            .filter_map(|Reverse((_, prefix))| state.txs.record_by_prefix(&prefix))
            .map(|record| Arc::clone(&record.tx))
            .collect();
        Ok(transactions)
    }
}
