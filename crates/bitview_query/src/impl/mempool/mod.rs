use std::sync::Arc;

use brk_mempool::ReadOnlyState;
pub use brk_mempool::{MempoolBlockSource, ResolvedMempoolBlockDiff};
use brk_types::{MempoolBlock, MempoolBlockHash, PreciseRecommendedFees, RecommendedFees, Txid};
use serde::{Serialize, Serializer};
use serde_json::to_vec;

use crate::{Error, Query, RepresentationId, Result};

pub mod rbf;

pub use rbf::ResolvedRbf;

/// A mempool view added to confirmed data, or none while no pool is at this publication (starting
/// up, a rollback, or the moment a block publishes before the mempool follows): the confirmed
/// data answers alone meanwhile.
pub(crate) fn anchored<T: Default>(view: brk_error::Result<T>) -> Result<T> {
    match view {
        Err(brk_error::Error::StateUpdating) => Ok(T::default()),
        view => Ok(view?),
    }
}

fn serialize_json<T: Serialize>(value: &T) -> (Vec<u8>, RepresentationId) {
    let bytes = to_vec(value).unwrap();
    let identity = RepresentationId::content(&bytes);
    (bytes, identity)
}

/// One publication's txid array, serialized only once a response needs it.
pub struct MempoolTxids {
    mempool: Arc<ReadOnlyState>,
    hash: u64,
}

impl MempoolTxids {
    /// The array's order-sensitive validator.
    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn json(&self) -> Vec<u8> {
        let txids = self.mempool.txids().expect("published when resolved");
        let mut bytes = Vec::new();
        serde_json::Serializer::new(&mut bytes)
            .collect_seq(txids)
            .expect("txids serialize");
        bytes
    }
}

impl Query {
    fn require_mempool(&self) -> Result<Arc<ReadOnlyState>> {
        self.mempool().ok_or(Error::MempoolNotAvailable)
    }

    /// Mempool statistics with their exact content identity, built once per publication.
    pub fn mempool_info_json(&self) -> Result<(Vec<u8>, RepresentationId)> {
        let (bytes, hash) = self.require_mempool()?.info_json()?;
        Ok((bytes.to_vec(), RepresentationId::Content(hash)))
    }

    /// The pool's txids from one publication: their validator now, the array on demand.
    pub fn mempool_txids(&self) -> Result<MempoolTxids> {
        let mempool = self.require_mempool()?;
        let hash = mempool.txids_hash()?;
        Ok(MempoolTxids { mempool, hash })
    }

    /// mempool.space's `/api/v1/fees/recommended`: whole sat/vB.
    pub fn recommended_fees(&self) -> Result<RecommendedFees> {
        Ok(self.require_mempool()?.fees()?)
    }

    /// mempool.space's `/api/v1/fees/precise`.
    pub fn precise_fees(&self) -> Result<PreciseRecommendedFees> {
        Ok(self.require_mempool()?.precise_fees()?)
    }

    pub fn mempool_blocks(&self) -> Result<Vec<MempoolBlock>> {
        let mempool = self.require_mempool()?;
        Ok(mempool
            .block_stats()?
            .iter()
            .map(MempoolBlock::from)
            .collect())
    }

    /// Serialize recent transactions with their exact content identity.
    pub fn mempool_recent_json(&self) -> Result<(Vec<u8>, RepresentationId)> {
        let recent = self.require_mempool()?.recent_txs()?;
        Ok(serialize_json(&recent))
    }

    /// Transaction times and an exact, order-sensitive result hash from one
    /// mempool state snapshot.
    pub fn transaction_times_with_hash(&self, txids: &[Txid]) -> Result<(Vec<u64>, u64)> {
        Ok(self.require_mempool()?.transaction_times_with_hash(txids)?)
    }

    /// Capture a published template for cheap validation and deferred construction.
    pub fn resolve_mempool_block(&self) -> Result<MempoolBlockSource> {
        let source = self.require_mempool()?.mempool_block_source();
        source.hash()?;
        Ok(source)
    }

    /// Validate history before conditional handling and capture both diff inputs.
    pub fn resolve_mempool_block_diff(
        &self,
        since: MempoolBlockHash,
    ) -> Result<ResolvedMempoolBlockDiff> {
        let mempool = self.require_mempool()?;
        let resolved = mempool
            .resolve_mempool_block_diff(since)
            .ok_or_else(|| Error::NotFound(format!("unknown since hash: {since}")))?;
        resolved.source().hash()?;
        Ok(resolved)
    }
}
