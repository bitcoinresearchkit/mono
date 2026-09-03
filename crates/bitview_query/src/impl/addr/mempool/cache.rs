use brk_mempool::Mempool;
use brk_types::AddrBytes;

use super::{AddrMempoolTxsJson, AddrMempoolTxsSource, ResolvedAddrMempoolTxs};
use crate::r#impl::{BoundedJsonCache, CachedJson};

const MAX_ENTRIES: usize = 256;
const MAX_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    addr: AddrBytes,
    limit: usize,
}

/// Exact serialized address responses with hard entry and byte bounds.
pub(crate) struct AddrMempoolTxsCache {
    empty: CachedJson,
    entries: BoundedJsonCache<CacheKey, AddrMempoolTxsSource>,
}

impl Default for AddrMempoolTxsCache {
    fn default() -> Self {
        Self {
            empty: CachedJson::from_slice(b"[]"),
            entries: BoundedJsonCache::new(MAX_ENTRIES, MAX_BYTES),
        }
    }
}

impl AddrMempoolTxsCache {
    pub(super) fn empty(&self) -> AddrMempoolTxsJson {
        AddrMempoolTxsJson::new(self.empty.value(), None)
    }

    pub(super) fn current(
        &self,
        addr: &AddrBytes,
        limit: usize,
        source: AddrMempoolTxsSource,
    ) -> Option<AddrMempoolTxsJson> {
        let key = CacheKey {
            addr: addr.clone(),
            limit,
        };
        self.entries
            .current(&key, &source)
            .map(|value| AddrMempoolTxsJson::new(value, Some(source)))
    }

    pub(super) fn get_or_build(
        &self,
        mempool: &Mempool,
        resolved: ResolvedAddrMempoolTxs,
    ) -> AddrMempoolTxsJson {
        let (addr, limit, expected_source) = resolved.into_parts();
        if let Some(cached) = self.current(&addr, limit, expected_source) {
            return cached;
        }

        let (transactions, revision) = mempool.addr_txs_with_revision(&addr, limit);
        let Some(revision) = revision else {
            return self.empty();
        };
        let source = (revision, transactions.len());
        let json = CachedJson::serialize(&transactions);
        if source != expected_source {
            return AddrMempoolTxsJson::new(json.value(), Some(source));
        }

        let key = CacheKey { addr, limit };
        let value = self.entries.insert(key, source, json);
        AddrMempoolTxsJson::new(value, Some(source))
    }
}
