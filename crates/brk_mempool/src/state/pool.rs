use std::{
    sync::{Arc, Mutex, OnceLock},
    time::Instant,
};

use brk_error::{Error, Result};
use brk_types::{BlockHash, FeeRate, MempoolInfo, Timestamp, Txid};

use crate::{
    JsonBody, PendingView, RbfNode, Snapshot, json,
    stores::{
        AddrTracker, OutpointSpends, ReadOnlyTxStore, TxGraveyard, TxStore, Upkeep,
        tx_store::TxRecord,
    },
};

use super::{Ancestry, State};

/// One reader tip's view past the pool's, once built: its readers wait for the first build.
pub type BehindSlot = Arc<Mutex<Option<Arc<PendingView>>>>;

/// One complete membership observation and its matching graph.
pub(crate) struct Pool {
    pub tip: BlockHash,
    /// `tip` and its newest ancestors.
    pub ancestry: Ancestry,
    pub info: MempoolInfo,
    pub txs: Arc<ReadOnlyTxStore>,
    pub addrs: Arc<AddrTracker>,
    pub outpoint_spends: Arc<OutpointSpends>,
    pub graveyard: Arc<TxGraveyard>,
    pub graph: Arc<Snapshot>,
    /// `info` as JSON and its hash, built by the writer: readers only clone the `Arc`.
    pub info_json: JsonBody,
    /// The txid array as JSON, built by the first request (a publication with the same members
    /// shares it too).
    pub txids_json: OnceLock<Arc<[u8]>>,
    /// The recently added transactions as JSON and its hash, built by the first request (a
    /// publication with the same contents shares it too).
    pub recent_json: OnceLock<JsonBody>,
    /// The recent replacement trees (all, then full-RBF only), walked by the first request.
    pub recent_trees: [OnceLock<Vec<RbfNode>>; 2],
    /// The recent listings as served, with the reader tip whose chain enriched them: the last
    /// one built, shared by every read of that moment.
    pub replacements_json: [Mutex<Option<(BlockHash, JsonBody)>>; 2],
    /// When it was published: of two saves of the graveyard, the later publication's wins.
    pub published_at: Instant,
    /// The views made for readers past this pool's tip (the moment after a block), one slot per
    /// reader tip, the newest few: each is built once, its readers sharing it.
    pub behind: Mutex<Vec<(BlockHash, BehindSlot)>>,
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
                json::body(&state.info),
            ),
        };
        let txids_json = previous
            .filter(|pool| pool.txs.membership_revision() == state.txs.membership_revision())
            .map_or_else(OnceLock::new, |pool| pool.txids_json.clone());
        let recent_json = previous
            .filter(|pool| pool.txs.content_revision() == state.txs.content_revision())
            .map_or_else(OnceLock::new, |pool| pool.recent_json.clone());
        // Shared while its mapping is unchanged.
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
        // The writer's ancestry when it has applied this tip's blocks, else just the tip.
        let ancestry = if state.ancestry.tip() == Some(&tip) {
            state.ancestry.clone()
        } else {
            Ancestry::of(tip)
        };
        Self {
            tip,
            ancestry,
            info: state.info.clone(),
            txs,
            addrs,
            outpoint_spends,
            graveyard,
            graph,
            info_json,
            txids_json,
            recent_json,
            recent_trees: Default::default(),
            replacements_json: Default::default(),
            published_at: Instant::now(),
            behind: Mutex::default(),
        }
    }

    /// A pending transaction's record and chunk rate, as a burial would keep them.
    pub fn pending_head(&self, txid: &Txid) -> Option<(&TxRecord, FeeRate)> {
        let record = self.txs.record(txid)?;
        let rate = self
            .graph
            .chunk_rate_for(txid)
            .unwrap_or_else(|| record.own_rate());
        Some((record, rate))
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
            return Err(Error::updating());
        }
        Ok(self)
    }

    pub fn first_seen(&self, txid: &Txid) -> Option<Timestamp> {
        self.txs
            .entry(txid)
            .map(|entry| entry.first_seen)
            .or_else(|| {
                self.graveyard
                    .get_unreplaced(txid)
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
            upkeep: Upkeep::of(&self.graveyard),
            ancestry: self.ancestry.clone(),
            // A panic may have abandoned a newer private witness: verify the restored records.
            unverified: self.txs.records().map(|(_, record)| record.entry.txid).collect(),
            template_only: Vec::new(),
        }
    }
}
