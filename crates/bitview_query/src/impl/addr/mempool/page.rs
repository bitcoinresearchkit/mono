use std::sync::Arc;

use crate::RepresentationId;

use super::AddrMempoolTxsSource;

/// One exact serialized address-mempool page and its internal source token.
pub(crate) struct AddrMempoolTxsJson {
    bytes: Arc<[u8]>,
    identity: RepresentationId,
    source: Option<AddrMempoolTxsSource>,
}

impl AddrMempoolTxsJson {
    pub(super) fn new(
        (bytes, identity): (Arc<[u8]>, RepresentationId),
        source: Option<AddrMempoolTxsSource>,
    ) -> Self {
        Self {
            bytes,
            identity,
            source,
        }
    }

    pub(crate) fn count(&self) -> usize {
        self.source.map(|(_, count)| count).unwrap_or_default()
    }

    pub(crate) fn source(&self) -> Option<AddrMempoolTxsSource> {
        self.source
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn into_value(self) -> (Arc<[u8]>, RepresentationId) {
        (self.bytes, self.identity)
    }
}
