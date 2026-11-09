use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode, header::ACCEPT_ENCODING},
    middleware::Next,
    response::{IntoResponse, Response},
};
use bitview_types::ErrorCode;

use crate::Error;

/// The server sends every response as it is, so one refusing `identity` gets 406, before the
/// handler runs (a broadcast would already be submitted).
pub async fn reject(request: Request<Body>, next: Next) -> Response {
    if refuses_identity(request.headers()) {
        return Error::new(
            StatusCode::NOT_ACCEPTABLE,
            ErrorCode::BadRequest,
            "No acceptable Content-Encoding: identity",
        )
        .into_response();
    }
    next.run(request).await
}

/// `identity;q=0`, or `*;q=0` without `identity` named (parameter names case-insensitive, RFC 9110
/// 12.4.2).
fn refuses_identity(headers: &HeaderMap) -> bool {
    let (mut named, mut any) = (None, None);
    for value in headers.get_all(ACCEPT_ENCODING) {
        for member in value.to_str().unwrap_or_default().split(',') {
            let mut parts = member.split(';').map(str::trim);
            let name = parts.next().unwrap_or_default();
            let zero = parts.any(|param| {
                param
                    .split_once('=')
                    .filter(|(key, _)| key.trim().eq_ignore_ascii_case("q"))
                    .and_then(|(_, q)| q.trim().parse::<f32>().ok())
                    .is_some_and(|q| q == 0.0)
            });
            if name.eq_ignore_ascii_case("identity") {
                named = Some(zero);
            } else if name == "*" {
                any = Some(zero);
            }
        }
    }
    named.or(any).unwrap_or(false)
}
