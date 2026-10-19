use std::sync::Arc;

use axum::{
    body::Bytes,
    http::{HeaderMap, HeaderValue, header},
    response::Response,
};
use bitview_query::{Error, Query, Result};
use tokio::{
    sync::Semaphore,
    time::{Instant, timeout_at},
};

use crate::{AppState, CacheParams, extended::ResponseExtended, raw_body::RawBodyPermit};

impl AppState {
    /// Selection, validator, and body belong to one read operation. The body takes a `bodies`
    /// permit before it is built and, when large, keeps it until its last frame is sent; without
    /// a free one, selection runs again once one is.
    pub async fn respond_admitted<S: 'static>(
        &self,
        admission: &Arc<Semaphore>,
        bodies: &Arc<Semaphore>,
        headers: HeaderMap,
        content_type: &'static str,
        select: impl FnOnce(&Query) -> Result<(S, CacheParams)> + Clone + Send + 'static,
        build: impl FnOnce(&Query, S) -> Result<Bytes> + Clone + Send + 'static,
    ) -> Result<Response> {
        let budget = bodies.clone();
        self.read_body(admission, bodies, move |q, permit| {
            let (source, params) = select(q)?;
            if params.etag.matches(&headers) {
                return Ok(Some(Response::new_not_modified(&params)));
            }
            let Some(permit) = permit.or_else(|| RawBodyPermit::try_acquire(&budget)) else {
                return Ok(None);
            };
            let bytes = build(q, source)?;
            Ok(Some(permit.response(params, bytes, |h| {
                h.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
            })))
        })
        .await
    }

    /// `None` requests response capacity, never an HTTP retry. The first
    /// attempt must check validators before trying capacity. Its guards are
    /// dropped before the wait; the second attempt resolves and validates anew.
    pub async fn read_body(
        &self,
        admission: &Arc<Semaphore>,
        bodies: &Arc<Semaphore>,
        build: impl FnOnce(&Query, Option<RawBodyPermit>) -> Result<Option<Response>>
        + Clone
        + Send
        + 'static,
    ) -> Result<Response> {
        let deadline = self.query.read_deadline();
        let query = self.query.with_deadline(deadline);
        let first = build.clone();
        if let Some(response) = query
            .read_with_admission(Some(admission), move |q| first(q, None))
            .await?
        {
            return Ok(response);
        }
        let permit = timeout_at(Instant::from_std(deadline), RawBodyPermit::acquire(bodies))
            .await
            .map_err(|_| Error::ReadTimeout)??;
        query
            .read_with_admission(Some(admission), move |q| build(q, Some(permit)))
            .await?
            .ok_or(Error::Internal(
                "admitted response requested capacity again",
            ))
    }
}
