use std::sync::Arc;

use axum::response::Response;
use bitview_query::{Error, Query, Result};
use tokio::{
    sync::Semaphore,
    time::{Instant, timeout_at},
};

use crate::{AppState, raw_body::RawBodyPermit};

impl AppState {
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
