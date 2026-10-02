use axum::{
    body::{Body, to_bytes},
    http::{
        Method, Request, Response, StatusCode,
        header::{ALLOW, CONTENT_TYPE},
    },
    middleware::Next,
    response::IntoResponse,
};

use crate::{error::Error, error_code::ErrorCode};

/// Maximum upstream error body buffered when constructing a JSON error.
const MAX_ERROR_BODY_BYTES: usize = 4096;

/// Matches `application/json` and `application/...+json`, ignoring parameters
/// like `; charset=utf-8`. Used to skip JSON-error rewriting for already-JSON bodies.
fn is_json_content_type(s: &str) -> bool {
    let mime = s.split(';').next().unwrap_or("").trim();
    mime == "application/json" || (mime.starts_with("application/") && mime.ends_with("+json"))
}

pub(crate) async fn respond(request: Request<Body>, next: Next) -> Response<Body> {
    let action = request.method() == Method::POST;
    let response = next.run(request).await;
    let status = response.status();
    if status.is_success()
        || status.is_redirection()
        || status.is_informational()
        || response
            .headers()
            .get(CONTENT_TYPE)
            .is_some_and(|v| v.to_str().is_ok_and(is_json_content_type))
    {
        return response;
    }

    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, MAX_ERROR_BODY_BYTES)
        .await
        .unwrap_or_default();
    let msg = String::from_utf8_lossy(&bytes);
    let (code, msg) = match parts.status {
        StatusCode::NOT_FOUND => (
            ErrorCode::NotFound,
            if msg.is_empty() {
                "Not found".into()
            } else {
                msg
            },
        ),
        StatusCode::METHOD_NOT_ALLOWED => (
            ErrorCode::MethodNotAllowed,
            "Method not allowed for this endpoint".into(),
        ),
        StatusCode::GATEWAY_TIMEOUT if action => (
            ErrorCode::Timeout,
            "Request timed out; submission outcome may be unknown".into(),
        ),
        StatusCode::GATEWAY_TIMEOUT => (ErrorCode::Timeout, "Request timed out".into()),
        s if s.is_client_error() => (
            ErrorCode::BadRequest,
            if msg.is_empty() {
                "Bad request".into()
            } else {
                msg
            },
        ),
        _ => (
            ErrorCode::InternalError,
            if msg.is_empty() {
                "Internal server error".into()
            } else {
                msg
            },
        ),
    };
    let msg = msg.into_owned();
    let mut response = Error::new(parts.status, code, msg).into_response();
    response.extensions_mut().extend(parts.extensions);
    if let Some(allow) = parts.headers.get(ALLOW) {
        response.headers_mut().insert(ALLOW, allow.clone());
    }
    response
}
