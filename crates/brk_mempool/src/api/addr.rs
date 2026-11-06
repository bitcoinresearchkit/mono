use std::{cmp::Reverse, collections::BinaryHeap};

use brk_error::{Error, Result};
use brk_types::{
    AddressBytes, AddressMempoolStats, BlockHash, Sats, TxOut, Txid, TxidPrefix, Vout,
};
use rapidhash::{RapidHashMap, RapidHashSet};
use rustc_hash::FxHashSet;

use crate::{
    PublishedTx, ReadOnlyState,
    key::{OutpointKey, TxKey},
};

/// The pool as a reader pinned at a chain tip sees it: at that tip, as published; behind it,
/// without what the chain's newer blocks settled: the transactions they mined, those they
/// conflicted (spent an input of) and those descending from a conflicted one.
pub struct PendingView {
    /// The pool's tip: the view reads only the publication it was made from.
    tip: BlockHash,
    settled: FxHashSet<TxKey>,
}

impl PendingView {
    fn pending(&self, txid: &Txid) -> bool {
        self.settled.is_empty() || !self.settled.contains(&TxKey::from(txid))
    }
}

/// An address's pending UTXO changes in one completed publication.
#[derive(Default)]
pub struct PendingUtxos {
    /// Outpoints the address's mempool transactions spend.
    pub spent: RapidHashSet<(Txid, Vout)>,
    /// Outputs mempool transactions pay the address that no mempool transaction spends, ordered
    /// by txid prefix and vout.
    pub created: Vec<(Txid, Vout, Sats)>,
    /// Identifies the address's mempool transactions: zero without any.
    pub token: u64,
}

impl ReadOnlyState {
    /// The pool for a reader at `tip`, when it's there.
    pub fn view_at(&self, tip: &BlockHash) -> Result<PendingView> {
        self.pool()?.at(tip)?;
        Ok(PendingView {
            tip: *tip,
            settled: FxHashSet::default(),
        })
    }

    /// The pool for a reader past its tip: `spends` are the newer blocks' inputs, each the
    /// outpoint it spends and its transaction. O(their inputs, the pooled spenders' inputs, and
    /// what they settle's outputs).
    pub fn view_behind(
        &self,
        spends: impl IntoIterator<Item = ((Txid, Vout), Txid)>,
    ) -> Result<PendingView> {
        let state = self.pool()?;
        let spends: RapidHashMap<(Txid, Vout), Txid> = spends.into_iter().collect();
        // The pooled spenders of those outpoints, each checked once against its own inputs: mined
        // when a block spends one of them by the transaction itself, conflicted when by another.
        let mut candidates: Vec<TxKey> = spends
            .keys()
            .filter_map(|&(txid, vout)| {
                state
                    .outpoint_spends
                    .get(&OutpointKey::new(TxKey::from(&txid), vout))
            })
            .collect();
        candidates.sort_unstable();
        candidates.dedup();
        let mut settled = FxHashSet::default();
        let mut conflicted = Vec::new();
        for record in candidates.iter().filter_map(|key| state.txs.record_by_key(key)) {
            let tx = &record.tx;
            let Some(spender) = tx
                .input
                .iter()
                .find_map(|input| spends.get(&(input.txid, input.vout)))
            else {
                continue;
            };
            settled.insert(record.entry.key());
            if *spender != tx.txid {
                conflicted.push(record);
            }
        }
        // What spends a conflicted transaction's outputs can't confirm either.
        while let Some(record) = conflicted.pop() {
            for child in state.outpoint_spends.children(&state.txs, &record.tx) {
                if settled.insert(child.entry.key()) {
                    conflicted.push(child);
                }
            }
        }
        Ok(PendingView {
            tip: state.tip,
            settled,
        })
    }

    /// The address's pending statistics.
    pub fn addr_stats(&self, addr: &AddressBytes, view: &PendingView) -> Result<AddressMempoolStats> {
        let state = self.pool()?.at(&view.tip)?;
        let Some(entry) = state.addrs.get(addr) else {
            return Ok(AddressMempoolStats::default());
        };
        if entry.txids.iter().all(|txid| view.pending(txid)) {
            return Ok(entry.stats.clone());
        }
        // Folded again without what the newer blocks settled.
        let pays = |txout: &TxOut| txout.addr_bytes().as_ref() == Some(addr);
        let mut stats = AddressMempoolStats::default();
        for txid in entry.txids.iter().filter(|txid| view.pending(txid)) {
            let Some(record) = state.txs.record(txid) else {
                continue;
            };
            for prevout in record.tx.input.iter().filter_map(|txin| txin.prevout.as_ref()) {
                if pays(prevout) {
                    stats.add_spend(prevout);
                }
            }
            for txout in record.tx.output.iter().filter(|txout| pays(txout)) {
                stats.add_funding(txout);
            }
            stats.tx_count += 1;
        }
        Ok(stats)
    }

    /// The address's pending spends and unspent pending outputs.
    pub fn addr_pending_utxos(&self, addr: &AddressBytes, view: &PendingView) -> Result<PendingUtxos> {
        let state = self.pool()?.at(&view.tip)?;
        let mut pending = PendingUtxos::default();
        let Some(entry) = state.addrs.get(addr) else {
            return Ok(pending);
        };
        let pays = |txout: &TxOut| txout.addr_bytes().as_ref() == Some(addr);
        for txid in entry.txids.iter().filter(|txid| view.pending(txid)) {
            let Some(record) = state.txs.record(txid) else {
                continue;
            };
            pending.token ^= *TxKey::from(txid);
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

    /// Whether `txid` is pending in `view`.
    pub fn is_pending(&self, txid: &Txid, view: &PendingView) -> Result<bool> {
        let state = self.pool()?.at(&view.tip)?;
        Ok(view.pending(txid) && state.txs.contains(txid))
    }

    /// Whether the pending tx `txid` touches `addr`: `StateUpdating` while it waits to fold in
    /// (what it touches isn't known yet).
    pub fn addr_has_tx(&self, addr: &AddressBytes, txid: &Txid, view: &PendingView) -> Result<bool> {
        let state = self.pool()?.at(&view.tip)?;
        if view.pending(txid) && state.addrs.is_waiting(txid) {
            return Err(Error::StateUpdating);
        }
        Ok(view.pending(txid)
            && state
                .addrs
                .get(addr)
                .is_some_and(|entry| entry.txids.contains(txid)))
    }

    /// Pending txs touching `addr`, newest first by `first_seen`, after the
    /// `after` cursor (older than it) or from the newest, capped at `limit`. A cursor
    /// that isn't one of the address's transactions has none after it. Shares immutable
    /// bodies from one completed publication; later prevout fills use copy-on-write and
    /// cannot mutate this selection.
    pub fn addr_txs(
        &self,
        addr: &AddressBytes,
        after: Option<&Txid>,
        limit: usize,
        view: &PendingView,
    ) -> Result<Vec<PublishedTx>> {
        let state = self.pool()?.at(&view.tip)?;
        let Some(entry) = state.addrs.get(addr) else {
            return Ok(Vec::new());
        };
        if limit == 0 {
            return Ok(Vec::new());
        }
        // Page order: the smallest timestamp is at the heap root; the txid prefix breaks
        // equal-time ties deterministically.
        let order = |txid: &Txid, first_seen| {
            Reverse((first_seen, TxidPrefix::from(txid), TxKey::from(txid)))
        };
        let after = match after {
            None => None,
            Some(txid) => match (entry.txids.contains(txid) && view.pending(txid))
                .then(|| state.txs.record(txid))
            {
                Some(Some(record)) => Some(order(txid, record.entry.first_seen)),
                _ => return Ok(Vec::new()),
            },
        };
        // Retain only the requested page.
        let mut newest = BinaryHeap::with_capacity(limit.min(entry.txids.len()));
        for txid in entry.txids.iter().filter(|txid| view.pending(txid)) {
            let Some(record) = state.txs.record(txid) else {
                continue;
            };
            let candidate = order(txid, record.entry.first_seen);
            if after.as_ref().is_some_and(|after| candidate <= *after) {
                continue;
            }
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
            .filter_map(|Reverse((_, _, key))| state.txs.record_by_key(&key))
            .map(PublishedTx::from_record)
            .collect();
        Ok(transactions)
    }
}
