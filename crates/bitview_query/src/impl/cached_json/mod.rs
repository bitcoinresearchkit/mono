use std::sync::Arc;

use serde::Serialize;

use crate::RepresentationId;

mod cache;

pub(crate) use cache::BoundedJsonCache;

/// Exact serialized JSON and its content-derived identity.
pub(crate) struct CachedJson {
    bytes: Arc<[u8]>,
    identity: RepresentationId,
}

impl CachedJson {
    pub(crate) fn serialize<T: Serialize>(value: &T) -> Self {
        Self::from_vec(serde_json::to_vec(value).unwrap())
    }

    pub(crate) fn from_slice(bytes: &[u8]) -> Self {
        Self::from_bytes(bytes.into())
    }

    pub(crate) fn from_vec(bytes: Vec<u8>) -> Self {
        Self::from_bytes(bytes.into())
    }

    fn from_bytes(bytes: Arc<[u8]>) -> Self {
        let identity = RepresentationId::content(&bytes);
        Self { bytes, identity }
    }

    pub(crate) fn len(&self) -> usize {
        self.bytes.len()
    }

    pub(crate) fn value(&self) -> (Arc<[u8]>, RepresentationId) {
        (Arc::clone(&self.bytes), self.identity)
    }
}
