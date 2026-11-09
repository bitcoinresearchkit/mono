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

/// One admitted response, from allocation until its body is sent.
#[derive(Clone)]
pub struct RawBodyPermit {
    _permit: Arc<OwnedSemaphorePermit>,
}

impl RawBodyPermit {
    // Eight concurrent responses per budget: raw blocks are at most 4 MB each (32 MB in all),
    // chain JSON can be larger.
    #[cfg(feature = "full-api")]
    pub const CAPACITY: usize = 8;

    /// Bodies at least this large keep their permit until sent; smaller ones release it once
    /// built, so slow clients of small responses can't hold every permit. Build concurrency
    /// stays bounded either way.
    const RETAINED_FROM: usize = 256 * 1024;

    pub fn retains(len: usize) -> bool {
        len >= Self::RETAINED_FROM
    }

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

    /// Retain admission (when large) until the bytes are sent.
    pub fn response(
        self,
        params: CacheParams,
        bytes: Bytes,
        content_headers: impl FnOnce(&mut HeaderMap),
    ) -> Response {
        let bytes = if Self::retains(bytes.len()) { self.bytes(bytes) } else { bytes };
        AppState::assemble_response(params, bytes, content_headers)
    }

    /// A body already built, on a budget small bodies share with large ones. A small one needs no
    /// permit, so clients stalling large responses can't hold it up; a large one takes one, `None`
    /// if none is free.
    pub fn respond_built(
        permit: Option<Self>,
        budget: &Arc<Semaphore>,
        params: CacheParams,
        bytes: Bytes,
        content_headers: impl FnOnce(&mut HeaderMap),
    ) -> Option<Response> {
        if !Self::retains(bytes.len()) {
            return Some(AppState::assemble_response(params, bytes, content_headers));
        }
        let permit = permit.or_else(|| Self::try_acquire(budget))?;
        Some(permit.response(params, bytes, content_headers))
    }

    /// [`Self::respond_built`] for a response already assembled.
    pub fn attach_built(
        permit: Option<Self>,
        budget: &Arc<Semaphore>,
        response: Response,
    ) -> Option<Response> {
        if Self::is_small(&response) {
            return Some(response);
        }
        Some(permit.or_else(|| Self::try_acquire(budget))?.attach(response))
    }

    /// Retain admission through a large response's body.
    fn attach(self, response: Response) -> Response {
        response.map(|body| Body::new(RetainedBody { body, permit: self }))
    }

    fn is_small(response: &Response) -> bool {
        response
            .body()
            .size_hint()
            .exact()
            .is_some_and(|len| !Self::retains(len as usize))
    }
}
