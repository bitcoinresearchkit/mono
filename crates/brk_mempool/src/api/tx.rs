//! Tx-keyed reads.

use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use brk_error::{Error, Result};
use brk_types::{
    BlockHash, MempoolRecentTx, OutpointPrefix, Transaction, TxOutspend, TxStatus, Txid,
    TxidPrefix, Vin, Vout,
};
use rustc_hash::{FxHashSet, FxHasher};

use crate::{ReadOnlyState, state::Pool};

fn transaction_times_hash(times: &[u64]) -> u64 {
    let mut hasher = FxHasher::default();
    times.hash(&mut hasher);
    hasher.finish()
}

impl ReadOnlyState {
    pub fn contains_txid(&self, txid: &Txid, tip: &BlockHash) -> Result<bool> {
        let state = self.pool()?;
        state.ensure_resolved_at(tip)?;
        Ok(state.txs.contains(txid))
    }

    /// Capture an immutable live body or its recently vanished fallback from
    /// one completed publication. Replaced tombstones are excluded.
    pub fn transaction(&self, txid: &Txid, tip: &BlockHash) -> Result<Option<Arc<Transaction>>> {
        let state = self.pool()?;
        state.ensure_resolved_at(tip)?;
        Ok(state
            .txs
            .record(txid)
            .map(|record| record.tx.clone())
            .or_else(|| {
                state
                    .graveyard
                    .get_vanished(txid)
                    .map(|tomb| tomb.tx.clone())
            }))
    }

    /// Spend status for a live mempool transaction's output, or `None` when
    /// the parent is not in the mempool. Parent presence, output bounds, and
    /// the spender are resolved from one publication.
    pub fn outspend_if_present(
        &self,
        txid: &Txid,
        vout: Vout,
        tip: &BlockHash,
    ) -> Result<Option<TxOutspend>> {
        let state = self.pool()?;
        state.ensure_resolved_at(tip)?;
        let Some(transaction) = state.txs.get(txid) else {
            return Ok(None);
        };
        if usize::from(vout) >= transaction.output.len() {
            return Ok(Some(TxOutspend::UNSPENT));
        }
        Ok(Some(Self::outspend_for(state, txid, vout)))
    }

    /// Current mempool spend status for an output whose parent may be
    /// confirmed. The spender's full input list is checked to rule out prefix
    /// collisions.
    pub fn outspend(&self, txid: &Txid, vout: Vout, tip: &BlockHash) -> Result<TxOutspend> {
        let state = self.pool()?;
        state.ensure_resolved_at(tip)?;
        Ok(Self::outspend_for(state, txid, vout))
    }

    /// Spend statuses for every output of a live mempool transaction, or
    /// `None` when the parent is not in the mempool. The complete array is
    /// resolved from one publication.
    pub fn outspends_if_present(
        &self,
        txid: &Txid,
        tip: &BlockHash,
    ) -> Result<Option<Vec<TxOutspend>>> {
        let state = self.pool()?;
        state.ensure_resolved_at(tip)?;
        let Some(transaction) = state.txs.get(txid) else {
            return Ok(None);
        };
        Self::outspends_for(state, txid, transaction.output.len()).map(Some)
    }

    /// Overlay mempool spends onto unresolved outputs of a confirmed parent.
    /// Returns immediately when every output is already confirmed spent.
    /// The complete overlay uses the acquired publication.
    pub fn merge_outspends(
        &self,
        txid: &Txid,
        outspends: &mut [TxOutspend],
        tip: &BlockHash,
    ) -> Result<()> {
        if outspends.iter().all(|outspend| outspend.spent) {
            return Ok(());
        }
        check_output_count(outspends.len())?;
        let state = self.pool()?;
        state.ensure_resolved_at(tip)?;
        Self::overlay_outspends(state, txid, outspends);
        Ok(())
    }

    fn outspend_for(state: &Pool, txid: &Txid, vout: Vout) -> TxOutspend {
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

    fn outspends_for(state: &Pool, txid: &Txid, output_count: usize) -> Result<Vec<TxOutspend>> {
        check_output_count(output_count)?;
        let mut outspends = vec![TxOutspend::UNSPENT; output_count];
        Self::overlay_outspends(state, txid, &mut outspends);
        Ok(outspends)
    }

    /// Resolve a spender's inputs once for the whole parent, retaining exact
    /// outpoint validation and never replacing a confirmed spend.
    fn overlay_outspends(state: &Pool, txid: &Txid, outspends: &mut [TxOutspend]) {
        let parent = TxidPrefix::from(txid);
        let mut scanned = FxHashSet::default();
        for index in 0..outspends.len() {
            if outspends[index].spent {
                continue;
            }
            let key = OutpointPrefix::new(parent, Vout::from(index));
            let Some(prefix) = state.outpoint_spends.get(&key) else {
                continue;
            };
            if !scanned.insert(prefix) {
                continue;
            }
            let Some(spender) = state.txs.record_by_prefix(&prefix) else {
                continue;
            };
            for (vin, input) in spender.tx.input.iter().enumerate() {
                if input.txid != *txid {
                    continue;
                }
                let Some(outspend) = outspends.get_mut(usize::from(input.vout)) else {
                    continue;
                };
                if !outspend.spent
                    && state
                        .outpoint_spends
                        .get(&OutpointPrefix::new(parent, input.vout))
                        == Some(prefix)
                {
                    *outspend = TxOutspend {
                        spent: true,
                        txid: Some(spender.entry.txid),
                        vin: Some(Vin::from(vin)),
                        status: Some(TxStatus::UNCONFIRMED),
                    };
                }
            }
        }
    }

    fn lookup_spender_for(state: &Pool, txid: &Txid, vout: Vout) -> Option<(Txid, Vin)> {
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

    /// Order-sensitive validator for the current txid array.
    pub fn txids_hash(&self) -> Result<u64> {
        let state = self.pool()?;
        Ok(state.txs.txids_hash())
    }

    /// Current txid array and its validator from one publication.
    pub fn txids_with_hash(&self) -> Result<(Vec<Txid>, u64)> {
        let state = self.pool()?;
        let txids = state.txs.txids().copied().collect();
        Ok((txids, state.txs.txids_hash()))
    }

    /// Recently observed additions, including transactions that have since left.
    pub fn recent_txs(&self) -> Result<Vec<MempoolRecentTx>> {
        let state = self.pool()?;
        Ok(state.txs.recent().to_vec())
    }

    /// Transaction times and an order-sensitive hash of that exact result,
    /// captured from one state snapshot.
    pub fn transaction_times_with_hash(&self, txids: &[Txid]) -> Result<(Vec<u64>, u64)> {
        let state = self.pool()?;
        let times = Self::transaction_times_for(state, txids);
        let hash = transaction_times_hash(&times);
        Ok((times, hash))
    }

    fn transaction_times_for(state: &Pool, txids: &[Txid]) -> Vec<u64> {
        txids
            .iter()
            .map(|txid| state.first_seen(txid).map_or(0, u64::from))
            .collect()
    }
}

fn check_output_count(count: usize) -> Result<()> {
    if count > usize::from(Vout::MAX) + 1 {
        return Err(Error::Internal(
            "Mempool output count exceeds index capacity",
        ));
    }
    Ok(())
}
