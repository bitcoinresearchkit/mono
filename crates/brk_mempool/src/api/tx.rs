use std::hash::{Hash, Hasher};

use brk_error::Result;
use brk_types::{BlockHash, MempoolRecentTx, TxOutspend, TxStatus, Txid, Vin, Vout};
use rustc_hash::{FxHashSet, FxHasher};

use crate::{
    PublishedTx, ReadOnlyState,
    key::{OutpointKey, TxKey},
    state::Pool,
};

impl ReadOnlyState {
    pub fn contains_txid(&self, txid: &Txid, tip: &BlockHash) -> Result<bool> {
        let state = self.pool()?.resolved_at(tip)?;
        Ok(state.txs.contains(txid))
    }

    /// Capture an immutable live body or its recently vanished fallback from
    /// one completed publication. Replaced tombstones are excluded.
    pub fn transaction(&self, txid: &Txid, tip: &BlockHash) -> Result<Option<PublishedTx>> {
        let state = self.pool()?.resolved_at(tip)?;
        Ok(state
            .txs
            .record(txid)
            .map(PublishedTx::from_record)
            .or_else(|| {
                state
                    .graveyard
                    .get_vanished(txid)
                    .map(|tomb| PublishedTx::from_record(&tomb.record))
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
        let state = self.pool()?.resolved_at(tip)?;
        let Some(transaction) = state.txs.get(txid) else {
            return Ok(None);
        };
        if usize::from(vout) >= transaction.output.len() {
            return Ok(Some(TxOutspend::UNSPENT));
        }
        Ok(Some(Self::outspend_for(state, txid, vout)))
    }

    /// Current mempool spend status for an output whose parent may be
    /// confirmed. The spender's full input list is checked to rule out key
    /// collisions.
    pub fn outspend(&self, txid: &Txid, vout: Vout, tip: &BlockHash) -> Result<TxOutspend> {
        let state = self.pool()?.resolved_at(tip)?;
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
        let state = self.pool()?.resolved_at(tip)?;
        let Some(transaction) = state.txs.get(txid) else {
            return Ok(None);
        };
        let mut outspends = vec![TxOutspend::UNSPENT; transaction.output.len()];
        Self::overlay_outspends(state, txid, &mut outspends);
        Ok(Some(outspends))
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
        let state = self.pool()?.resolved_at(tip)?;
        Self::overlay_outspends(state, txid, outspends);
        Ok(())
    }

    fn outspend_for(state: &Pool, txid: &Txid, vout: Vout) -> TxOutspend {
        let Some((spender, vin)) = state.outpoint_spends.spender(&state.txs, txid, vout) else {
            return TxOutspend::UNSPENT;
        };
        TxOutspend {
            spent: true,
            txid: Some(spender.entry.txid),
            vin: Some(vin),
            status: Some(TxStatus::UNCONFIRMED),
        }
    }

    /// Resolve a spender's inputs once for the whole parent, retaining exact
    /// outpoint validation and never replacing a confirmed spend.
    fn overlay_outspends(state: &Pool, txid: &Txid, outspends: &mut [TxOutspend]) {
        let parent = TxKey::from(txid);
        let mut scanned = FxHashSet::default();
        for index in 0..outspends.len() {
            if outspends[index].spent {
                continue;
            }
            let outpoint = OutpointKey::new(parent, Vout::from(index));
            let Some(spender_key) = state.outpoint_spends.get(&outpoint) else {
                continue;
            };
            if !scanned.insert(spender_key) {
                continue;
            }
            let Some(spender) = state.txs.record_by_key(&spender_key) else {
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
                        .get(&OutpointKey::new(parent, input.vout))
                        == Some(spender_key)
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

    /// Order-sensitive validator for the current txid array.
    pub fn txids_hash(&self) -> Result<u64> {
        let state = self.pool()?;
        Ok(state.txs.txids_hash())
    }

    /// The txid array [`Self::txids_hash`] validates.
    pub fn txids(&self) -> Result<impl Iterator<Item = &Txid>> {
        Ok(self.pool()?.txs.txids())
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
        let times: Vec<u64> = txids
            .iter()
            .map(|txid| state.first_seen(txid).map_or(0, u64::from))
            .collect();
        let mut hasher = FxHasher::default();
        times.as_slice().hash(&mut hasher);
        Ok((times, hasher.finish()))
    }
}
