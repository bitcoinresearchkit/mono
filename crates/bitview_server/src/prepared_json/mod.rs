use axum::{body::Bytes, http::HeaderMap, response::Response};
use bitview_query::RepresentationId;

use crate::{CacheParams, extended::ResponseExtended};

/// Server-lifetime JSON with its validator computed once from the body.
pub struct PreparedJson {
    bytes: Bytes,
    params: CacheParams,
}

impl PreparedJson {
    #[cfg(any(feature = "series", feature = "chain"))]
    pub fn new(value: impl serde::Serialize) -> Self {
        Self::from_bytes(Bytes::from(serde_json::to_vec(&value).unwrap()))
    }

    pub fn from_bytes(bytes: Bytes) -> Self {
        let hash = RepresentationId::content_hash(&bytes);
        let params = CacheParams::revalidate(format!("c{hash:x}"));
        Self { bytes, params }
    }

    pub fn respond(&self, headers: &HeaderMap) -> Response {
        Response::cached_bytes(headers, &self.params, "application/json", || {
            self.bytes.clone()
        })
    }

    #[cfg(feature = "series")]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
