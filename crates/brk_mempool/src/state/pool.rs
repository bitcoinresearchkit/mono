use std::{hash::Hasher, sync::Arc};

use brk_error::{Error, Result};
use brk_types::{BlockHash, MempoolInfo, Timestamp, Txid};
use rustc_hash::{FxHashSet, FxHasher};
use serde_json::to_vec;

use crate::{
    Snapshot,
    stores::{AddrTracker, OutpointSpends, ReadOnlyTxStore, TxGraveyard, TxStore},
};

use super::State;

/// One complete membership observation and its matching graph.
pub(crate) struct Pool {
    pub tip: BlockHash,
    pub info: MempoolInfo,
    pub txs: Arc<ReadOnlyTxStore>,
    pub addrs: Arc<AddrTracker>,
    pub outpoint_spends: Arc<OutpointSpends>,
    pub graveyard: Arc<TxGraveyard>,
    pub graph: Arc<Snapshot>,
    /// `info` as JSON and its hash, built by the writer: readers only clone the `Arc`.
    pub info_json: (Arc<[u8]>, u64),
    inputs_complete: bool,
}

impl Pool {
    pub fn new(
        state: &State,
        tip: BlockHash,
        graph: Arc<Snapshot>,
        previous: Option<&Self>,
    ) -> Self {
        debug_assert_eq!(graph.content_revision(), state.txs.content_revision());
        // A new projection or chain tip does not change membership containers.
        let (txs, addrs, info_json) = match previous
            .filter(|pool| pool.txs.content_revision() == state.txs.content_revision())
        {
            Some(pool) => (pool.txs.clone(), pool.addrs.clone(), pool.info_json.clone()),
            None => (
                Arc::new(state.txs.read_only_clone()),
                Arc::new(state.addrs.clone()),
                Self::info_json(&state.info),
            ),
        };
        // Spend ownership can change alone (a stale template tx turning listed).
        let outpoint_spends = previous
            .filter(|pool| pool.outpoint_spends.revision() == state.outpoint_spends.revision())
            .map_or_else(
                || Arc::new(state.outpoint_spends.clone()),
                |pool| pool.outpoint_spends.clone(),
            );
        let graveyard = previous
            .filter(|pool| pool.graveyard.revision() == state.graveyard.revision())
            .map_or_else(
                || Arc::new(state.graveyard.clone()),
                |pool| pool.graveyard.clone(),
            );
        Self {
            tip,
            info: state.info.clone(),
            txs,
            addrs,
            outpoint_spends,
            graveyard,
            graph,
            info_json,
            inputs_complete: state.txs.unresolved().is_empty(),
        }
    }

    fn info_json(info: &MempoolInfo) -> (Arc<[u8]>, u64) {
        let bytes = to_vec(info).expect("mempool info serializes");
        let mut hasher = FxHasher::default();
        hasher.write(&bytes);
        (bytes.into(), hasher.finish())
    }

    pub fn matches(&self, state: &State, tip: &BlockHash, graph: &Arc<Snapshot>) -> bool {
        self.tip == *tip
            && self.txs.content_revision() == state.txs.content_revision()
            && self.outpoint_spends.revision() == state.outpoint_spends.revision()
            && self.graveyard.revision() == state.graveyard.revision()
            && Arc::ptr_eq(&self.graph, graph)
    }

    /// This publication, when it is anchored at `tip`.
    pub fn at(&self, tip: &BlockHash) -> Result<&Self> {
        if self.tip != *tip {
            return Err(Error::StateUpdating);
        }
        Ok(self)
    }

    /// This publication, when it is anchored at `tip` with every input resolved.
    pub fn resolved_at(&self, tip: &BlockHash) -> Result<&Self> {
        if !self.inputs_complete {
            return Err(Error::StateUpdating);
        }
        self.at(tip)
    }

    pub fn first_seen(&self, txid: &Txid) -> Option<Timestamp> {
        self.txs
            .entry(txid)
            .map(|entry| entry.first_seen)
            .or_else(|| {
                self.graveyard
                    .get_vanished(txid)
                    .map(|tomb| tomb.record.entry.first_seen)
            })
    }

    /// Restart private work from a known complete publication after a panic.
    pub fn restore(&self) -> State {
        State {
            info: self.info.clone(),
            txs: TxStore::from_read_only(self.txs.as_ref().clone()),
            addrs: self.addrs.as_ref().clone(),
            outpoint_spends: self.outpoint_spends.as_ref().clone(),
            graveyard: self.graveyard.as_ref().clone(),
            template_only: FxHashSet::default(),
        }
    }
}
