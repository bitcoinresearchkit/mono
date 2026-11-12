//! HTTP mechanics shared by Bitview's servers: `If-None-Match` matching and the lengthless
//! `304 Not Modified`, the browser's and the CDN's cache policies, and media-type matching.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    body::{Body, Bytes, HttpBody},
    http::{
        HeaderMap, HeaderName, HeaderValue, Response, StatusCode,
        header::{CACHE_CONTROL, IF_NONE_MATCH},
    },
};
use http_body::Frame;

/// The CDN's own cache policy, ahead of `Cache-Control` for it.
pub const CDN_CACHE_CONTROL: HeaderName = HeaderName::from_static("cdn-cache-control");

/// Reused for a second, then asked again, never served stale: a browser's and a CDN's policy for
/// what may change at any time (an error's verdict included, with a release; browsers are never
/// purged).
pub const REVALIDATE: &str = "public, max-age=1, must-revalidate";

/// Held up to a year and never revalidated: bytes only a new address (a version in it) or a
/// purge replaces.
pub const IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// How an answer without an ETag may be cached, the browser's and the CDN's alike: an error (a
/// conditional request for one gets the error again, not a 304), or an answer reused for a second
/// at most.
#[derive(Clone, Copy)]
pub enum CachePolicy {
    /// [`REVALIDATE`].
    Revalidate,
    /// Never stored: an action's answer, a transient or private one, or one that depends on the
    /// request.
    NoStore,
}

impl CachePolicy {
    /// Sets it as the browser's and the CDN's policy.
    pub fn apply(self, headers: &mut HeaderMap) {
        let policy = match self {
            Self::Revalidate => REVALIDATE,
            Self::NoStore => "no-store",
        };
        insert_cache_policy(headers, policy, policy);
    }

    /// Sets it on an error answer and records it there, so a layer that rewrites the error's body
    /// keeps it: for an answer whose cacheability its handler knows (one that varies by request).
    pub fn mark(self, response: &mut Response<Body>) {
        self.apply(response.headers_mut());
        response.extensions_mut().insert(self);
    }
}

/// Sets the browser's cache policy (`Cache-Control`) and the CDN's.
pub fn insert_cache_policy(headers: &mut HeaderMap, browser: &'static str, cdn: &'static str) {
    headers.insert(CACHE_CONTROL, HeaderValue::from_static(browser));
    headers.insert(CDN_CACHE_CONTROL, HeaderValue::from_static(cdn));
}

/// A media type without its parameters (`; charset=utf-8`): what a `Content-Type`, or one range of
/// an `Accept`, names.
pub fn media_type(value: &str) -> &str {
    value.split_once(';').map_or(value, |(media, _)| media).trim()
}

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
                quoted.toggle();
            }
            byte == b',' && !quoted
        })
        .filter_map(|entry| {
            let entry = entry.trim_ascii();
            let entry = entry.trim_prefix(b"W/");
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
    type Error = !;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, !>>> {
        Poll::Ready(None)
    }

    fn is_end_stream(&self) -> bool {
        true
    }
}
