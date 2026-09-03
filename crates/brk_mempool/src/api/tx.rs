//! Tx-keyed reads.

use std::hash::{Hash, Hasher};

use brk_types::{
    MempoolRecentTx, OutpointPrefix, Transaction, TxOutspend, TxStatus, Txid, TxidPrefix, Vin, Vout,
};
use rustc_hash::FxHasher;

use crate::{Mempool, State};

fn transaction_times_hash(times: &[u64]) -> u64 {
    let mut hasher = FxHasher::default();
    times.hash(&mut hasher);
    hasher.finish()
}

impl Mempool {
    pub fn contains_txid(&self, txid: &Txid) -> bool {
        self.read().txs.contains(txid)
    }

    /// Apply `f` to the live tx body if present.
    pub fn with_tx<R>(&self, txid: &Txid, f: impl FnOnce(&Transaction) -> R) -> Option<R> {
        self.read().txs.get(txid).map(f)
    }

    /// Apply `f` to a `Vanished` tombstone's tx body if present.
    /// `Replaced` tombstones return `None` because the tx will not confirm.
    pub fn with_vanished_tx<R>(&self, txid: &Txid, f: impl FnOnce(&Transaction) -> R) -> Option<R> {
        self.read().graveyard.get_vanished(txid).map(|t| f(&t.tx))
    }

    /// Spend status for a live mempool transaction's output, or `None` when
    /// the parent is not in the mempool. Parent presence, output bounds, and
    /// the spender are resolved under one state read lock.
    pub fn outspend_if_present(&self, txid: &Txid, vout: Vout) -> Option<TxOutspend> {
        let state = self.read();
        let transaction = state.txs.get(txid)?;
        if usize::from(vout) >= transaction.output.len() {
            return Some(TxOutspend::UNSPENT);
        }
        Some(Self::outspend_for(&state, txid, vout))
    }

    /// Current mempool spend status for an output whose parent may be
    /// confirmed. The spender's full input list is checked to rule out prefix
    /// collisions.
    pub fn outspend(&self, txid: &Txid, vout: Vout) -> TxOutspend {
        Self::outspend_for(&self.read(), txid, vout)
    }

    /// Spend statuses for every output of a live mempool transaction, or
    /// `None` when the parent is not in the mempool. The complete array is
    /// resolved under one state read lock.
    pub fn outspends_if_present(&self, txid: &Txid) -> Option<Vec<TxOutspend>> {
        let state = self.read();
        let output_count = state.txs.get(txid)?.output.len();
        Some(Self::outspends_for(&state, txid, output_count))
    }

    /// Overlay mempool spends onto unresolved outputs of a confirmed parent.
    /// Returns without taking the state lock when every output is already
    /// confirmed spent; otherwise the complete overlay uses one read lock.
    pub fn merge_outspends(&self, txid: &Txid, outspends: &mut [TxOutspend]) {
        if outspends.iter().all(|outspend| outspend.spent) {
            return;
        }
        let state = self.read();
        for (index, outspend) in outspends.iter_mut().enumerate() {
            if !outspend.spent {
                *outspend = Self::outspend_for(&state, txid, Vout::from(index));
            }
        }
    }

    /// Mempool tx spending `(txid, vout)`, or `None`. The spender's
    /// input list is walked to rule out `TxidPrefix` collisions.
    pub fn lookup_spender(&self, txid: &Txid, vout: Vout) -> Option<(Txid, Vin)> {
        Self::lookup_spender_for(&self.read(), txid, vout)
    }

    fn outspend_for(state: &State, txid: &Txid, vout: Vout) -> TxOutspend {
        let Some((spender_txid, vin)) = Self::lookup_spender_for(state, txid, vout) else {
            return TxOutspend::UNSPENT;
        };
        TxOutspend {
            spent: true,
            txid: Some(spender_txid),
            vin: Some(vin),
            status: Some(TxStatus::UNCONFIRMED),
        }
    }

    fn outspends_for(state: &State, txid: &Txid, output_count: usize) -> Vec<TxOutspend> {
        (0..output_count)
            .map(|index| Self::outspend_for(state, txid, Vout::from(index)))
            .collect()
    }

    fn lookup_spender_for(state: &State, txid: &Txid, vout: Vout) -> Option<(Txid, Vin)> {
        let key = OutpointPrefix::new(TxidPrefix::from(txid), vout);
        let spender = state
            .outpoint_spends
            .get(&key)
            .and_then(|prefix| state.txs.record_by_prefix(&prefix))?;
        let vin_pos = spender
            .tx
            .input
            .iter()
            .position(|input| input.txid == *txid && input.vout == vout)?;
        Some((spender.entry.txid, Vin::from(vin_pos)))
    }

    /// Snapshot of all live mempool txids.
    ///
    /// Allocates `32 * len(mempool)` bytes under the read guard. Sized for
    /// diagnostics. Route layers serving large pools should paginate at
    /// their boundary rather than calling this per request.
    #[must_use]
    pub fn txids(&self) -> Vec<Txid> {
        self.read().txs.txids().copied().collect()
    }

    /// Order-sensitive validator for the current txid array.
    #[must_use]
    pub fn txids_hash(&self) -> u64 {
        self.read().txs.txids_hash()
    }

    /// Current txid array and its validator from one state-lock window.
    #[must_use]
    pub fn txids_with_hash(&self) -> (Vec<Txid>, u64) {
        let state = self.read();
        let txids = state.txs.txids().copied().collect();
        (txids, state.txs.txids_hash())
    }

    /// Snapshot of recent live txs.
    #[must_use]
    pub fn recent_txs(&self) -> Vec<MempoolRecentTx> {
        self.read().txs.recent().to_vec()
    }

    /// `first_seen` Unix-second timestamps for `txids`, in input order.
    /// Returns 0 for unknown txids. `Vanished` tombstones fall back to
    /// the buried entry's `first_seen` to avoid flicker between drop
    /// and indexer catch-up.
    #[must_use]
    pub fn transaction_times(&self, txids: &[Txid]) -> Vec<u64> {
        let state = self.read();
        Self::transaction_times_for(&state, txids)
    }

    /// Transaction times and an order-sensitive hash of that exact result,
    /// captured from one state snapshot.
    #[must_use]
    pub fn transaction_times_with_hash(&self, txids: &[Txid]) -> (Vec<u64>, u64) {
        let state = self.read();
        let times = Self::transaction_times_for(&state, txids);
        let hash = transaction_times_hash(&times);
        (times, hash)
    }

    fn transaction_times_for(state: &State, txids: &[Txid]) -> Vec<u64> {
        txids
            .iter()
            .map(|txid| state.first_seen(txid).map_or(0, u64::from))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use brk_types::Vout;

    use super::transaction_times_hash;
    use crate::{
        Mempool,
        state::TxEntry,
        test_support::{fake_entry_info, fake_tx, fake_txid, p2wpkh_script},
    };

    #[test]
    fn outspend_resolves_parent_and_spender_from_one_snapshot() {
        let mempool = Mempool::for_test();
        let parent = fake_tx(
            1,
            &[],
            &[(p2wpkh_script(1), 1_000), (p2wpkh_script(2), 2_000)],
        );
        let mut spender = fake_tx(2, &[None], &[(p2wpkh_script(2), 900)]);
        spender.input[0].txid = parent.txid;
        spender.input[0].vout = Vout::ZERO;

        let parent_entry = TxEntry::new(&fake_entry_info(parent.txid, 100, 100), 100, false);
        let spender_entry = TxEntry::new(&fake_entry_info(spender.txid, 100, 100), 100, false);
        let mut state = mempool.test_state_lock().write();
        state
            .outpoint_spends
            .insert_spends(&spender, spender_entry.txid_prefix());
        state.txs.insert(parent.clone(), parent_entry);
        state.txs.insert(spender.clone(), spender_entry);
        drop(state);

        let outspend = mempool
            .outspend_if_present(&parent.txid, Vout::ZERO)
            .unwrap();
        assert!(outspend.spent);
        assert_eq!(outspend.txid, Some(spender.txid));
        assert_eq!(outspend.vin, Some(0usize.into()));
        assert!(outspend.status.is_some_and(|status| !status.confirmed));

        let outspends = mempool.outspends_if_present(&parent.txid).unwrap();
        assert_eq!(outspends.len(), 2);
        assert!(outspends[0].spent);
        assert!(!outspends[1].spent);

        assert!(
            !mempool
                .outspend_if_present(&parent.txid, Vout::from(2usize))
                .unwrap()
                .spent
        );
        assert!(
            mempool
                .outspend_if_present(&fake_txid(3), Vout::ZERO)
                .is_none()
        );
    }

    #[test]
    fn transaction_times_hash_includes_order_and_length() {
        assert_ne!(
            transaction_times_hash(&[1, 2]),
            transaction_times_hash(&[2, 1])
        );
        assert_ne!(
            transaction_times_hash(&[1]),
            transaction_times_hash(&[1, 0])
        );
        assert_eq!(
            transaction_times_hash(&[1, 2]),
            transaction_times_hash(&[1, 2])
        );
    }
}
