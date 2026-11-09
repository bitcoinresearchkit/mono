use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use bitview_types::ErrorCode;

use crate::Error;

/// The API sends every response as it is, so one refusing `identity` gets 406, before the handler
/// runs (a broadcast would already be submitted).
pub async fn reject(request: Request<Body>, next: Next) -> Response {
    if bitview_http::accepts_encoding(request.headers(), "identity") == Some(false) {
        return Error::new(
            StatusCode::NOT_ACCEPTABLE,
            ErrorCode::BadRequest,
            "No acceptable Content-Encoding: identity",
        )
        .into_response();
    }
    next.run(request).await
}
