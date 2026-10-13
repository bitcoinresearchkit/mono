//! Address-keyed reads.

use std::{cmp::Reverse, collections::BinaryHeap, sync::Arc};

use brk_error::Result;
use brk_types::{
    AddressBytes, AddressMempoolStats, BlockHash, Sats, Transaction, TxOut, Txid, TxidPrefix, Vout,
};
use rustc_hash::FxHashSet;

use crate::ReadOnlyState;

/// An address's pending UTXO changes in one completed publication.
#[derive(Default)]
pub struct PendingUtxos {
    /// Outpoints the address's mempool transactions spend.
    pub spent: FxHashSet<(Txid, Vout)>,
    /// Outputs mempool transactions pay the address that no mempool transaction spends, by txid.
    pub created: Vec<(Txid, Vout, Sats)>,
    /// Identifies the address's mempool transactions: zero without any.
    pub token: u64,
}

impl ReadOnlyState {
    /// Statistics from a completed publication anchored to the requested chain.
    pub fn addr_stats(&self, addr: &AddressBytes, tip: &BlockHash) -> Result<AddressMempoolStats> {
        let state = self.pool()?;
        state.ensure_resolved_at(tip)?;
        Ok(state
            .addrs
            .get(addr)
            .map(|entry| entry.stats.clone())
            .unwrap_or_default())
    }

    /// The address's pending spends and unspent pending outputs.
    pub fn addr_pending_utxos(&self, addr: &AddressBytes, tip: &BlockHash) -> Result<PendingUtxos> {
        let state = self.pool()?;
        state.ensure_resolved_at(tip)?;
        let mut pending = PendingUtxos::default();
        let Some(entry) = state.addrs.get(addr) else {
            return Ok(pending);
        };
        let pays = |txout: &TxOut| txout.addr_bytes().as_ref() == Some(addr);
        for txid in &entry.txids {
            let Some(record) = state.txs.record(txid) else {
                continue;
            };
            pending.token ^= *TxidPrefix::from(txid);
            let tx = &record.tx;
            for txin in &tx.input {
                if txin.prevout.as_ref().is_some_and(pays) {
                    pending.spent.insert((txin.txid, txin.vout));
                }
            }
            for (vout, txout) in tx.output.iter().enumerate() {
                if pays(txout) {
                    pending
                        .created
                        .push((tx.txid, Vout::from(vout), txout.value));
                }
            }
        }
        pending
            .created
            .retain(|(txid, vout, _)| !pending.spent.contains(&(*txid, *vout)));
        pending
            .created
            .sort_unstable_by_key(|(txid, vout, _)| (TxidPrefix::from(txid), *vout));
        Ok(pending)
    }

    /// Live mempool txs touching `addr`, newest first by `first_seen`,
    /// capped at `limit`. Shares immutable bodies from one completed publication;
    /// later prevout fills use copy-on-write and cannot mutate this selection.
    pub fn addr_txs(
        &self,
        addr: &AddressBytes,
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
