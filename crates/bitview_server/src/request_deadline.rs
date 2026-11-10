use std::time::Instant;

use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tokio::time::{self, Instant as TimeInstant};

use crate::{Error, REQUEST_TIMEOUT};
use crate::request_action::RequestAction;

/// The one deadline shared by HTTP handling and its blocking query work.
#[derive(Clone, Copy)]
pub(crate) struct RequestDeadline(pub Instant);

pub async fn apply(mut request: Request<Body>, next: Next) -> Response {
    let deadline = Instant::now() + REQUEST_TIMEOUT;
    request.extensions_mut().insert(RequestDeadline(deadline));
    let action = request.extensions().get::<RequestAction>().cloned()
        .unwrap_or_else(|| RequestAction::new(request.method()));
    match time::timeout_at(TimeInstant::from_std(deadline), next.run(request)).await {
        Ok(response) => response,
        Err(_) => Error::timeout(action.is_action()).into_response(),
    }
}
