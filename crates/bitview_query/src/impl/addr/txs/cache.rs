use std::sync::Arc;

use brk_types::{AddrBytes, BlockHash};

use crate::{
    RepresentationId,
    r#impl::{BoundedJsonCache, CachedJson, addr::mempool::AddrMempoolTxsSource},
};

use super::combined::AddrTxsLimits;

const MAX_ENTRIES: usize = 256;
const MAX_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    addr: AddrBytes,
    mempool_limit: usize,
    chain_floor: usize,
    total_target: usize,
}

#[derive(PartialEq, Eq)]
struct CacheSource {
    mempool: AddrMempoolTxsSource,
    chain: BlockHash,
}

/// Exact serialized mixed address responses with hard entry and byte bounds.
pub(crate) struct AddrTxsCache(BoundedJsonCache<CacheKey, CacheSource>);

impl Default for AddrTxsCache {
    fn default() -> Self {
        Self(BoundedJsonCache::new(MAX_ENTRIES, MAX_BYTES))
    }
}

impl AddrTxsCache {
    pub(super) fn current(
        &self,
        addr: &AddrBytes,
        limits: AddrTxsLimits,
        mempool: AddrMempoolTxsSource,
        chain: BlockHash,
    ) -> Option<(Arc<[u8]>, RepresentationId)> {
        self.0.current(
            &CacheKey {
                addr: addr.clone(),
                mempool_limit: limits.mempool_limit,
                chain_floor: limits.chain_floor,
                total_target: limits.total_target,
            },
            &CacheSource { mempool, chain },
        )
    }

    pub(super) fn insert(
        &self,
        addr: AddrBytes,
        limits: AddrTxsLimits,
        mempool: AddrMempoolTxsSource,
        chain: BlockHash,
        json: CachedJson,
    ) -> (Arc<[u8]>, RepresentationId) {
        self.0.insert(
            CacheKey {
                addr,
                mempool_limit: limits.mempool_limit,
                chain_floor: limits.chain_floor,
                total_target: limits.total_target,
            },
            CacheSource { mempool, chain },
            json,
        )
    }
}
