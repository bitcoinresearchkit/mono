use std::sync::Arc;

use brk_error::{Error, Result};
use brk_mempool::{Mempool, ResolvedBlockTemplateDiff};
use brk_types::{
    BlockTemplate, BlockTemplateDiff, MempoolBlock, MempoolInfo, MempoolRecentTx, NextBlockHash,
    RecommendedFees, Txid,
};
use serde::Serialize;

mod block_template;
mod rbf;

pub(crate) use block_template::BlockTemplateCache;
pub use block_template::BlockTemplateDiffPreflight;
pub use rbf::ResolvedRbf;

use crate::{Query, RepresentationId, representation_id::content_hash};

fn serialize_json<T: Serialize>(value: &T) -> (Vec<u8>, RepresentationId) {
    let bytes = serde_json::to_vec(value).unwrap();
    let identity = RepresentationId::Content(content_hash(&bytes));
    (bytes, identity)
}

impl Query {
    fn require_mempool(&self) -> Result<&Mempool> {
        self.mempool().ok_or(Error::MempoolNotAvailable)
    }

    pub fn mempool_info(&self) -> Result<MempoolInfo> {
        Ok(self.require_mempool()?.info())
    }

    /// Serialize mempool statistics with their exact content identity.
    pub fn mempool_info_json(&self) -> Result<(Vec<u8>, RepresentationId)> {
        let info = self.mempool_info()?;
        Ok(serialize_json(&info))
    }

    pub fn mempool_txids(&self) -> Result<Vec<Txid>> {
        Ok(self.require_mempool()?.txids())
    }

    pub fn mempool_txids_hash(&self) -> Result<u64> {
        Ok(self.require_mempool()?.txids_hash())
    }

    pub fn mempool_txids_with_hash(&self) -> Result<(Vec<Txid>, u64)> {
        Ok(self.require_mempool()?.txids_with_hash())
    }

    pub fn recommended_fees(&self) -> Result<RecommendedFees> {
        self.require_mempool().map(|m| m.fees())
    }

    pub fn mempool_blocks(&self) -> Result<Vec<MempoolBlock>> {
        let mempool = self.require_mempool()?;
        Ok(mempool
            .block_stats()
            .iter()
            .map(MempoolBlock::from)
            .collect())
    }

    pub fn mempool_recent(&self) -> Result<Vec<MempoolRecentTx>> {
        Ok(self.require_mempool()?.recent_txs())
    }

    /// Serialize recent transactions with their exact content identity.
    pub fn mempool_recent_json(&self) -> Result<(Vec<u8>, RepresentationId)> {
        let recent = self.mempool_recent()?;
        Ok(serialize_json(&recent))
    }

    /// `first_seen` Unix-second timestamps. Matches mempool.space's
    /// `POST /api/v1/transaction-times`. Returns 0 for unknowns.
    pub fn transaction_times(&self, txids: &[Txid]) -> Result<Vec<u64>> {
        Ok(self.require_mempool()?.transaction_times(txids))
    }

    /// Transaction times and an exact, order-sensitive result hash from one
    /// mempool state snapshot.
    pub fn transaction_times_with_hash(&self, txids: &[Txid]) -> Result<(Vec<u64>, u64)> {
        Ok(self.require_mempool()?.transaction_times_with_hash(txids))
    }

    /// Content hash of the projected next block. Polling lets monitors detect
    /// a stalled sync.
    pub fn mempool_hash(&self) -> Result<NextBlockHash> {
        Ok(self.require_mempool()?.next_block_hash())
    }

    /// Full projected next block (Core's `getblocktemplate` selection)
    /// with stats and full tx bodies in GBT order.
    pub fn block_template(&self) -> Result<BlockTemplate> {
        Ok(self.require_mempool()?.block_template())
    }

    /// Return the current serialized template without waiting for an in-flight build.
    pub fn block_template_json_cached(&self) -> Result<Option<(Arc<[u8]>, RepresentationId)>> {
        let mempool = self.require_mempool()?;
        Ok(self.0.block_template_cache.current(mempool))
    }

    /// Return the current serialized template, building it at most once per source revision.
    pub fn block_template_json(&self) -> Result<(Arc<[u8]>, RepresentationId)> {
        let mempool = self.require_mempool()?;
        Ok(self.0.block_template_cache.get_or_build(mempool))
    }

    /// Resolve an exact cached diff or capture its historical input once.
    pub fn block_template_diff_json_preflight(
        &self,
        since: NextBlockHash,
    ) -> Result<BlockTemplateDiffPreflight> {
        let mempool = self.require_mempool()?;
        if let Some((bytes, identity)) = self.0.block_template_cache.current_diff(mempool, since) {
            return Ok(BlockTemplateDiffPreflight::Cached(bytes, identity));
        }
        let resolved = mempool
            .resolve_block_template_diff(since)
            .ok_or_else(|| Error::NotFound(format!("unknown since hash: {since}")))?;
        Ok(BlockTemplateDiffPreflight::Resolved(resolved))
    }

    /// Serialize a diff from history captured by `block_template_diff_json_preflight`.
    pub fn block_template_diff_json_resolved(
        &self,
        resolved: ResolvedBlockTemplateDiff,
    ) -> Result<(Arc<[u8]>, RepresentationId)> {
        let mempool = self.require_mempool()?;
        Ok(self
            .0
            .block_template_cache
            .get_or_build_diff(mempool, resolved))
    }

    /// Delta of the projected next block since `since`. `NotFound`
    /// when `since` has aged out (client should fall back to
    /// `block_template`).
    pub fn block_template_diff(&self, since: NextBlockHash) -> Result<BlockTemplateDiff> {
        self.require_mempool()?
            .block_template_diff(since)
            .ok_or_else(|| Error::NotFound(format!("unknown since hash: {since}")))
    }
}
