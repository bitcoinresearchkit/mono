use std::{str::FromStr, sync::Arc};

use brk_error::{Error, Result};
use brk_mempool::Mempool;
use brk_types::{Addr, AddrBytes, Transaction};

mod cache;
mod page;
mod resolved;

pub(crate) use cache::AddrMempoolTxsCache;
pub(crate) use page::AddrMempoolTxsJson;
pub use resolved::ResolvedAddrMempoolTxs;

use crate::{Query, RepresentationId};

pub(crate) type AddrMempoolTxsSource = (u64, usize);

pub(crate) enum AddrMempoolTxsPagePreflight {
    Cached(AddrMempoolTxsJson),
    Resolved(ResolvedAddrMempoolTxs),
}

impl AddrMempoolTxsPagePreflight {
    pub(crate) fn count(&self) -> usize {
        match self {
            Self::Cached(json) => json.count(),
            Self::Resolved(resolved) => resolved.source().1,
        }
    }

    pub(crate) fn source(&self) -> Option<AddrMempoolTxsSource> {
        match self {
            Self::Cached(json) => json.source(),
            Self::Resolved(resolved) => Some(resolved.source()),
        }
    }
}

pub enum AddrMempoolTxsPreflight {
    Cached(Arc<[u8]>, RepresentationId),
    Resolved(ResolvedAddrMempoolTxs),
}

impl AddrMempoolTxsPreflight {
    fn cached((bytes, identity): (Arc<[u8]>, RepresentationId)) -> Self {
        Self::Cached(bytes, identity)
    }
}

impl Query {
    pub fn addr_mempool_txs(&self, addr: &Addr, limit: usize) -> Result<Vec<Transaction>> {
        let bytes = AddrBytes::from_str(addr)?;
        let mempool = self.mempool().ok_or(Error::MempoolNotAvailable)?;
        Ok(mempool.addr_txs(&bytes, limit))
    }

    /// Resolve an invalid, empty, cached, or cold address response before body loading.
    pub fn addr_mempool_txs_json_preflight(
        &self,
        addr: &Addr,
        limit: usize,
    ) -> Result<AddrMempoolTxsPreflight> {
        let addr = AddrBytes::from_str(addr)?;
        let mempool = self.mempool().ok_or(Error::MempoolNotAvailable)?;
        match self.addr_mempool_txs_json_preflight_for(mempool, addr, limit) {
            AddrMempoolTxsPagePreflight::Cached(json) => {
                Ok(AddrMempoolTxsPreflight::cached(json.into_value()))
            }
            AddrMempoolTxsPagePreflight::Resolved(resolved) => {
                Ok(AddrMempoolTxsPreflight::Resolved(resolved))
            }
        }
    }

    /// Build and cache a response captured by [`Self::addr_mempool_txs_json_preflight`].
    pub fn addr_mempool_txs_json_resolved(
        &self,
        resolved: ResolvedAddrMempoolTxs,
    ) -> Result<(Arc<[u8]>, RepresentationId)> {
        self.addr_mempool_txs_json_resolved_page(resolved)
            .map(AddrMempoolTxsJson::into_value)
    }

    pub(crate) fn addr_mempool_txs_json_preflight_for(
        &self,
        mempool: &Mempool,
        addr: AddrBytes,
        limit: usize,
    ) -> AddrMempoolTxsPagePreflight {
        let Some(source) = mempool.addr_txs_source(&addr, limit) else {
            return AddrMempoolTxsPagePreflight::Cached(self.0.addr_mempool_txs_cache.empty());
        };
        if let Some(cached) = self.0.addr_mempool_txs_cache.current(&addr, limit, source) {
            return AddrMempoolTxsPagePreflight::Cached(cached);
        }
        AddrMempoolTxsPagePreflight::Resolved(ResolvedAddrMempoolTxs::new(addr, limit, source))
    }

    pub(crate) fn addr_mempool_txs_json_resolved_page(
        &self,
        resolved: ResolvedAddrMempoolTxs,
    ) -> Result<AddrMempoolTxsJson> {
        let mempool = self.mempool().ok_or(Error::MempoolNotAvailable)?;
        Ok(self
            .0
            .addr_mempool_txs_cache
            .get_or_build(mempool, resolved))
    }
}
