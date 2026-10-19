//! Conditional-request mechanics shared by Bitview's HTTP servers: `If-None-Match` matching and
//! the lengthless `304 Not Modified`.

use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    body::{Body, Bytes, HttpBody},
    http::{HeaderMap, Response, StatusCode, header::IF_NONE_MATCH},
};
use http_body::Frame;

/// Whether `If-None-Match` matches the current representation: `*` always does, and `etag` (its
/// opaque tag without quotes, `None` for an untagged one) compares weakly with every listed tag,
/// as a CDN may weaken or list them.
pub fn if_none_match(headers: &HeaderMap, etag: Option<&[u8]>) -> bool {
    headers.get_all(IF_NONE_MATCH).iter().any(|value| {
        let value = value.as_bytes();
        value.trim_ascii() == b"*"
            || etag.is_some_and(|etag| field_etags(value).any(|listed| listed == etag))
    })
}

/// The opaque tags `If-None-Match` lists, weak or strong, without their quotes.
pub fn listed_etags(headers: &HeaderMap) -> impl Iterator<Item = &[u8]> {
    headers
        .get_all(IF_NONE_MATCH)
        .iter()
        .flat_map(|value| field_etags(value.as_bytes()))
}

/// The opaque tags of one field value: a comma inside quotes belongs to its tag.
fn field_etags(value: &[u8]) -> impl Iterator<Item = &[u8]> {
    let mut quoted = false;
    value
        .split(move |&byte| {
            if byte == b'"' {
                quoted = !quoted;
            }
            byte == b',' && !quoted
        })
        .filter_map(|entry| {
            let entry = entry.trim_ascii();
            let entry = entry.strip_prefix(b"W/").unwrap_or(entry);
            entry.strip_prefix(b"\"")?.strip_suffix(b"\"")
        })
}

/// An empty `304 Not Modified`, its validator and caching headers left to the caller. Its body
/// gives Axum no exact length to turn into `Content-Length: 0`: a 304's Content-Length would have
/// to describe the full response.
pub fn not_modified() -> Response<Body> {
    let mut response = Response::new(Body::new(NotModifiedBody));
    *response.status_mut() = StatusCode::NOT_MODIFIED;
    response
}

struct NotModifiedBody;

impl HttpBody for NotModifiedBody {
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Infallible>>> {
        Poll::Ready(None)
    }

    fn is_end_stream(&self) -> bool {
        true
    }
}
