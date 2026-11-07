use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use brk_error::{Error, Result};
use brk_types::{BlockHash, MempoolRecentTx, TxOutspend, TxStatus, Txid, Vin, Vout};
use rustc_hash::{FxHashSet, FxHasher};
use serde::Serializer;

use crate::{
    PendingView, PublishedEntry, PublishedTx, ReadOnlyState, TxRemoval,
    key::{OutpointKey, TxKey},
    state::Pool,
};

impl ReadOnlyState {
    /// Why a transaction left the pool at `tip`, while the graveyard remembers it: `None` for one
    /// pending or unknown.
    pub fn removal(&self, txid: &Txid, tip: &BlockHash) -> Result<Option<TxRemoval>> {
        let state = self.pool()?.at(tip)?;
        Ok(state.graveyard.get(txid).map(|tomb| tomb.removal))
    }

    /// A pending transaction's entry facts at `tip` (Core's fee, vsize and first sighting): what
    /// needs no input resolved.
    pub fn entry(&self, txid: &Txid, tip: &BlockHash) -> Result<Option<PublishedEntry>> {
        let state = self.pool()?.at(tip)?;
        Ok(state.txs.entry(txid).map(|entry| PublishedEntry {
            fee: entry.fee,
            vsize: entry.vsize,
            first_seen: entry.first_seen,
        }))
    }

    /// The txid array [`Self::txids_hash`] validates.
    pub fn txids(&self) -> Result<impl Iterator<Item = &Txid>> {
        Ok(self.pool()?.txs.txids())
    }

    /// A transaction pending in `view`, from one completed publication: none for one that left
    /// (mined ones the chain answers; evicted, expired or replaced ones are gone). A body whose
    /// own inputs aren't all resolved yet answers `StateUpdating`, alone: others read meanwhile.
    pub fn transaction(&self, txid: &Txid, view: &PendingView) -> Result<Option<PublishedTx>> {
        let state = self.pool()?.at(view.tip())?;
        let Some(record) = state.txs.record(txid).filter(|_| view.pending(txid)) else {
            return Ok(None);
        };
        if !record.tx.is_resolved() {
            return Err(Error::StateUpdating);
        }
        Ok(Some(PublishedTx::from_record(record)))
    }

    /// Spend status for a pending transaction's output, or `None` when the parent isn't pending
    /// in `view`. Parent presence, output bounds, and the spender are resolved from one
    /// publication.
    pub fn outspend_if_present(
        &self,
        txid: &Txid,
        vout: Vout,
        view: &PendingView,
    ) -> Result<Option<TxOutspend>> {
        // Spends are known from inputs' outpoints: no prevout needs resolving.
        let state = self.pool()?.at(view.tip())?;
        let Some(transaction) = state.txs.get(txid).filter(|_| view.pending(txid)) else {
            return Ok(None);
        };
        if usize::from(vout) >= transaction.output.len() {
            return Ok(Some(TxOutspend::UNSPENT));
        }
        Ok(Some(Self::outspend_for(state, txid, vout, view)))
    }

    /// Pending spend status for an output whose parent may be confirmed. The spender's full
    /// input list is checked to rule out key collisions.
    pub fn outspend(&self, txid: &Txid, vout: Vout, view: &PendingView) -> Result<TxOutspend> {
        let state = self.pool()?.at(view.tip())?;
        Ok(Self::outspend_for(state, txid, vout, view))
    }

    /// Spend statuses for every output of a pending transaction, or `None` when the parent
    /// isn't pending in `view`. The complete array is resolved from one publication.
    pub fn outspends_if_present(
        &self,
        txid: &Txid,
        view: &PendingView,
    ) -> Result<Option<Vec<TxOutspend>>> {
        let state = self.pool()?.at(view.tip())?;
        let Some(transaction) = state.txs.get(txid).filter(|_| view.pending(txid)) else {
            return Ok(None);
        };
        let mut outspends = vec![TxOutspend::UNSPENT; transaction.output.len()];
        Self::overlay_outspends(state, txid, &mut outspends, view);
        Ok(Some(outspends))
    }

    /// Overlay pending spends onto unresolved outputs of a confirmed parent.
    /// Returns immediately when every output is already confirmed spent.
    /// The complete overlay uses the acquired publication.
    pub fn merge_outspends(
        &self,
        txid: &Txid,
        outspends: &mut [TxOutspend],
        view: &PendingView,
    ) -> Result<()> {
        if outspends.iter().all(|outspend| outspend.spent) {
            return Ok(());
        }
        let state = self.pool()?.at(view.tip())?;
        Self::overlay_outspends(state, txid, outspends, view);
        Ok(())
    }

    /// A spender the newer blocks settled (`view`) spends nothing any more.
    fn outspend_for(state: &Pool, txid: &Txid, vout: Vout, view: &PendingView) -> TxOutspend {
        let Some((spender, vin)) = state
            .outpoint_spends
            .spender(&state.txs, txid, vout)
            .filter(|(spender, _)| view.pending(&spender.entry.txid))
        else {
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
    fn overlay_outspends(
        state: &Pool,
        txid: &Txid,
        outspends: &mut [TxOutspend],
        view: &PendingView,
    ) {
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
            let Some(spender) = state
                .txs
                .record_by_key(&spender_key)
                .filter(|spender| view.pending(&spender.entry.txid))
            else {
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

    /// The current txid array's validator: the pool's content revision, which every insert and
    /// removal moves (so every change of the array, order included), made unique across restarts.
    pub fn txids_hash(&self) -> Result<u64> {
        let state = self.pool()?;
        Ok(state.txs.content_revision() ^ crate::json::process_nonce())
    }

    /// The txid array [`Self::txids_hash`] validates, as JSON: encoded once per publication.
    pub fn txids_json(&self) -> Result<Arc<[u8]>> {
        let state = self.pool()?;
        Ok(state
            .txids_json
            .get_or_init(|| {
                let mut bytes = Vec::new();
                serde_json::Serializer::new(&mut bytes)
                    .collect_seq(state.txs.txids())
                    .expect("txids serialize");
                bytes.into()
            })
            .clone())
    }

    /// Recently observed additions, including transactions that have since left.
    pub fn recent_txs(&self) -> Result<Vec<MempoolRecentTx>> {
        let state = self.pool()?;
        Ok(state.txs.recent().cloned().collect())
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
