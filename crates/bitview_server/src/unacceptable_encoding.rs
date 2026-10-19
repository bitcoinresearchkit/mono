use std::convert::Infallible;

use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode, header::ACCEPT_ENCODING},
    middleware::Next,
    response::{IntoResponse, Response},
};
use bitview_types::ErrorCode;
use tower::{ServiceExt, service_fn};
use tower_layer::Layer;

use crate::{Error, compression_layer, extended::HeaderMapExtended};

/// The compression layer answers 406 only after the handler ran (a broadcast would already be
/// submitted), so its own negotiation runs first, on an empty response.
pub async fn reject(request: Request<Body>, next: Next) -> Response {
    if refuses_an_encoding(request.headers()) && !negotiates(request.headers()).await {
        let mut response = Error::new(
            StatusCode::NOT_ACCEPTABLE,
            ErrorCode::BadRequest,
            "No acceptable Content-Encoding: identity, br, gzip or zstd",
        )
        .into_response();
        response.headers_mut().insert_vary_accept_encoding();
        return response;
    }
    next.run(request).await
}

/// Negotiation can only fail on a `q=0` member.
fn refuses_an_encoding(headers: &HeaderMap) -> bool {
    headers.get_all(ACCEPT_ENCODING).iter().any(|value| {
        value
            .as_bytes()
            .windows(3)
            .any(|w| w.eq_ignore_ascii_case(b"q=0"))
    })
}

async fn negotiates(headers: &HeaderMap) -> bool {
    let mut probe = Request::new(Body::empty());
    for value in headers.get_all(ACCEPT_ENCODING) {
        probe.headers_mut().append(ACCEPT_ENCODING, value.clone());
    }
    let empty = service_fn(|_| async { Ok::<_, Infallible>(Response::new(Body::empty())) });
    compression_layer()
        .layer(empty)
        .oneshot(probe)
        .await
        .is_ok_and(|response| response.status() != StatusCode::NOT_ACCEPTABLE)
}
