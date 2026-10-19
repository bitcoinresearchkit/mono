use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, HeaderValue, Response, header},
};
use serde::Serialize;
use serde_json::to_vec;

use super::header_map::HeaderMapExtended;
use crate::cache::CacheParams;

pub trait ResponseExtended
where
    Self: Sized,
{
    fn new_not_modified(params: &CacheParams) -> Self;
    /// `304` when the request's validator matches `params`; otherwise the body `bytes` builds,
    /// only then, as `content_type`.
    fn cached_bytes(
        headers: &HeaderMap,
        params: &CacheParams,
        content_type: &'static str,
        bytes: impl FnOnce() -> Bytes,
    ) -> Self;

    /// [`Self::cached_bytes`] for the JSON of the value `value` builds.
    fn cached_json<T: Serialize>(
        headers: &HeaderMap,
        params: &CacheParams,
        value: impl FnOnce() -> T,
    ) -> Self {
        Self::cached_bytes(headers, params, "application/json", || {
            Bytes::from(to_vec(&value()).unwrap())
        })
    }
}

impl ResponseExtended for Response<Body> {
    fn new_not_modified(params: &CacheParams) -> Response<Body> {
        let mut response = bitview_http::not_modified();
        let headers = response.headers_mut();
        headers.insert_vary_accept_encoding();
        params.apply_to(headers);
        response
    }

    fn cached_bytes(
        headers: &HeaderMap,
        params: &CacheParams,
        content_type: &'static str,
        bytes: impl FnOnce() -> Bytes,
    ) -> Self {
        if params.etag.matches(headers) {
            return Self::new_not_modified(params);
        }
        let mut response = Response::new(Body::from(bytes()));
        let h = response.headers_mut();
        h.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
        params.apply_to(h);
        response
    }
}
