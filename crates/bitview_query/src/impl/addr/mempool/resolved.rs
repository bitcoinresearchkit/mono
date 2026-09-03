use brk_types::AddrBytes;

use super::AddrMempoolTxsSource;

/// A live address response bound to one process-local mempool revision.
pub struct ResolvedAddrMempoolTxs {
    addr: AddrBytes,
    limit: usize,
    source: AddrMempoolTxsSource,
}

impl ResolvedAddrMempoolTxs {
    pub(super) fn new(addr: AddrBytes, limit: usize, source: AddrMempoolTxsSource) -> Self {
        Self {
            addr,
            limit,
            source,
        }
    }

    pub(super) fn source(&self) -> AddrMempoolTxsSource {
        self.source
    }

    pub(super) fn into_parts(self) -> (AddrBytes, usize, AddrMempoolTxsSource) {
        (self.addr, self.limit, self.source)
    }
}
