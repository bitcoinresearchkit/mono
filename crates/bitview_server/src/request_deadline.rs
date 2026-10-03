use std::time::{Duration, Instant};

use axum::{
    body::Body,
    http::{Method, Request},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tokio::time::{self, Instant as TimeInstant};

use crate::{Error, REQUEST_TIMEOUT};

/// The one deadline shared by HTTP handling and its blocking query work.
#[derive(Clone, Copy)]
pub(crate) struct RequestDeadline(pub Instant);

pub async fn apply(request: Request<Body>, next: Next) -> Response {
    apply_for(request, next, REQUEST_TIMEOUT).await
}

async fn apply_for(mut request: Request<Body>, next: Next, budget: Duration) -> Response {
    let deadline = Instant::now() + budget;
    request.extensions_mut().insert(RequestDeadline(deadline));
    let action = request.method() == Method::POST;
    match time::timeout_at(TimeInstant::from_std(deadline), next.run(request)).await {
        Ok(response) => response,
        Err(_) => Error::timeout(action).into_response(),
    }
}
