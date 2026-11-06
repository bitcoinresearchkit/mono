use std::sync::Arc;

use brk_error::{Error, Result};
use brk_types::{BlockHash, Txid, Vout};
use rapidhash::RapidHashMap;
use rustc_hash::FxHashSet;

use crate::{
    ReadOnlyState,
    key::{OutpointKey, TxKey},
};

/// The pool as a reader pinned at a chain tip sees it: at that tip, as published; behind it,
/// without what the chain's newer blocks settled: the transactions they mined, those they
/// conflicted (spent an input of) and those descending from a conflicted one.
pub struct PendingView {
    /// The pool's tip: the view reads only the publication it was made from.
    tip: BlockHash,
    /// The reader's.
    reader: BlockHash,
    settled: FxHashSet<TxKey>,
}

impl PendingView {
    pub(crate) fn tip(&self) -> &BlockHash {
        &self.tip
    }

    pub(crate) fn settles_nothing(&self) -> bool {
        self.settled.is_empty()
    }

    /// Whether `txid`, pooled, is still pending as the reader sees it.
    pub(crate) fn pending(&self, txid: &Txid) -> bool {
        self.pending_key(&TxKey::from(txid))
    }

    /// [`Self::pending`] by the transaction's key.
    pub(crate) fn pending_key(&self, key: &TxKey) -> bool {
        self.settled.is_empty() || !self.settled.contains(key)
    }
}

impl ReadOnlyState {
    /// The pool for a reader at `tip`, when it's there.
    pub fn view_at(&self, tip: &BlockHash) -> Result<Arc<PendingView>> {
        self.pool()?.at(tip)?;
        Ok(Arc::new(PendingView {
            tip: *tip,
            reader: *tip,
            settled: FxHashSet::default(),
        }))
    }

    /// The pool for a reader at `reader`, past its tip: `spends` gives the newer blocks' inputs,
    /// each the outpoint it spends and its transaction. Made once per publication and reader tip,
    /// under the pool's lock: every read of the moment after a block shares one (those arriving
    /// meanwhile wait for it). O(those inputs, the pooled spenders' inputs, and what they settle's
    /// outputs).
    pub fn view_behind<E: From<Error>, S: IntoIterator<Item = ((Txid, Vout), Txid)>>(
        &self,
        reader: BlockHash,
        spends: impl FnOnce() -> std::result::Result<S, E>,
    ) -> std::result::Result<Arc<PendingView>, E> {
        let state = self.pool()?;
        let mut behind = state
            .behind
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(view) = behind.as_ref().filter(|view| view.reader == reader) {
            return Ok(view.clone());
        }
        let spends: RapidHashMap<(Txid, Vout), Txid> = spends()?.into_iter().collect();
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
        let view = Arc::new(PendingView {
            tip: state.tip,
            reader,
            settled,
        });
        *behind = Some(view.clone());
        Ok(view)
    }
}
