use aide::OperationInput;
use axum::{extract::FromRequestParts, http::request::Parts};

use crate::Error;

/// Extractor that rejects requests carrying any query string.
/// Used on path-only endpoints to prevent cache-busting via injected
/// query params (the cache key includes the URI).
pub struct Empty;

impl<S> FromRequestParts<S> for Empty
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match parts.uri.query() {
            Some(q) if !q.is_empty() => Err(Error::bad_request(format!(
                "this endpoint does not accept query parameters (got `?{q}`)"
            ))),
            _ => Ok(Empty),
        }
    }
}

impl OperationInput for Empty {}
