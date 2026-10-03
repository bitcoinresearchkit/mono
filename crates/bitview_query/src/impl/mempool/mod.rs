use std::sync::Arc;

use brk_mempool::ReadOnlyState;
pub use brk_mempool::{BlockTemplateSource, ResolvedBlockTemplateDiff};
use brk_types::{MempoolBlock, MempoolInfo, MempoolRecentTx, NextBlockHash, RecommendedFees, Txid};
use serde::Serialize;
use serde_json::to_vec;

use crate::{Error, Query, RepresentationId, Result};

pub mod rbf;

pub use rbf::ResolvedRbf;

fn serialize_json<T: Serialize>(value: &T) -> (Vec<u8>, RepresentationId) {
    let bytes = to_vec(value).unwrap();
    let identity = RepresentationId::Content(RepresentationId::content_hash(&bytes));
    (bytes, identity)
}

impl Query {
    fn require_mempool(&self) -> Result<Arc<ReadOnlyState>> {
        self.mempool().ok_or(Error::MempoolNotAvailable)
    }

    fn mempool_info(&self) -> Result<MempoolInfo> {
        Ok(self.require_mempool()?.info()?)
    }

    /// Serialize mempool statistics with their exact content identity.
    pub fn mempool_info_json(&self) -> Result<(Vec<u8>, RepresentationId)> {
        let info = self.mempool_info()?;
        Ok(serialize_json(&info))
    }

    pub fn mempool_txids_hash(&self) -> Result<u64> {
        Ok(self.require_mempool()?.txids_hash()?)
    }

    pub fn mempool_txids_with_hash(&self) -> Result<(Vec<Txid>, u64)> {
        Ok(self.require_mempool()?.txids_with_hash()?)
    }

    pub fn recommended_fees(&self) -> Result<RecommendedFees> {
        Ok(self.require_mempool()?.fees()?)
    }

    pub fn mempool_blocks(&self) -> Result<Vec<MempoolBlock>> {
        let mempool = self.require_mempool()?;
        Ok(mempool
            .block_stats()?
            .iter()
            .map(MempoolBlock::from)
            .collect())
    }

    fn mempool_recent(&self) -> Result<Vec<MempoolRecentTx>> {
        Ok(self.require_mempool()?.recent_txs()?)
    }

    /// Serialize recent transactions with their exact content identity.
    pub fn mempool_recent_json(&self) -> Result<(Vec<u8>, RepresentationId)> {
        let recent = self.mempool_recent()?;
        Ok(serialize_json(&recent))
    }

    /// Transaction times and an exact, order-sensitive result hash from one
    /// mempool state snapshot.
    pub fn transaction_times_with_hash(&self, txids: &[Txid]) -> Result<(Vec<u64>, u64)> {
        Ok(self.require_mempool()?.transaction_times_with_hash(txids)?)
    }

    /// Content identity of the published projected next block, not a liveness clock.
    pub fn mempool_hash(&self) -> Result<NextBlockHash> {
        Ok(self.require_mempool()?.next_block_hash()?)
    }

    /// Capture a published template for cheap validation and deferred construction.
    pub fn resolve_block_template(&self) -> Result<BlockTemplateSource> {
        let source = self.require_mempool()?.block_template_source();
        source.hash()?;
        Ok(source)
    }

    /// Validate history before conditional handling and capture both diff inputs.
    pub fn resolve_block_template_diff(
        &self,
        since: NextBlockHash,
    ) -> Result<ResolvedBlockTemplateDiff> {
        let mempool = self.require_mempool()?;
        let resolved = mempool
            .resolve_block_template_diff(since)
            .ok_or_else(|| Error::NotFound(format!("unknown since hash: {since}")))?;
        resolved.source().hash()?;
        Ok(resolved)
    }
}
