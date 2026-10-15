use std::sync::Arc;

use axum::{
    body::{Body, Bytes, HttpBody},
    http::HeaderMap,
    response::Response,
};
use bitview_query::{Error, Result};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::{AppState, CacheParams};
use body::RetainedBody;
use bytes::RetainedBytes;

mod body;
mod bytes;

/// One admitted encoded response, from allocation through the final body/frame owner.
#[derive(Clone)]
pub struct RawBodyPermit {
    _permit: Arc<OwnedSemaphorePermit>,
}

impl RawBodyPermit {
    // At most 32 MB of raw serialized payloads (8 * the 4 MB block bound),
    // plus bounded-per-response encoder state and one admitted decoder.
    #[cfg(feature = "chain")]
    pub const CAPACITY: usize = 8;

    /// Bodies at least this large keep their permit until sent; smaller ones release it once
    /// built, so slow clients of small responses can't hold every permit. Build concurrency
    /// stays bounded either way.
    const RETAINED_FROM: usize = 256 * 1024;

    pub fn try_acquire(budget: &Arc<Semaphore>) -> Option<Self> {
        budget.clone().try_acquire_owned().ok().map(|permit| Self {
            _permit: Arc::new(permit),
        })
    }

    pub async fn acquire(budget: &Arc<Semaphore>) -> Result<Self> {
        let permit = budget
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| Error::Internal("response admission closed"))?;
        Ok(Self {
            _permit: Arc::new(permit),
        })
    }

    pub fn bytes(&self, bytes: Bytes) -> Bytes {
        Bytes::from_owner(RetainedBytes {
            bytes,
            _permit: self.clone(),
        })
    }

    /// Retain admission (when large) through both the raw bytes and any outer encoder.
    pub fn response(
        self,
        params: CacheParams,
        bytes: Bytes,
        content_headers: impl FnOnce(&mut HeaderMap),
    ) -> Response {
        if bytes.len() < Self::RETAINED_FROM {
            return AppState::assemble_response(params, bytes, content_headers);
        }
        let response = AppState::assemble_response(params, self.bytes(bytes), content_headers);
        self.attach(response)
    }

    /// Retain admission (when large) through a built response's body and any outer encoder.
    pub fn attach(self, mut response: Response) -> Response {
        let small = response
            .body()
            .size_hint()
            .exact()
            .is_some_and(|len| len < Self::RETAINED_FROM as u64);
        if !small {
            response.extensions_mut().insert(self);
        }
        response
    }

    /// Called outside compression so encoder state and emitted frames retain
    /// admission too, even after the encoder consumes its original raw input.
    pub fn retain(mut response: Response) -> Response {
        let Some(permit) = response.extensions_mut().remove::<Self>() else {
            return response;
        };
        response.map(|body| Body::new(RetainedBody { body, permit }))
    }
}
