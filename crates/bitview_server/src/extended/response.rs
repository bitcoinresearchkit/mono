use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, HeaderValue, Response, StatusCode, header},
};

use super::{header_map::HeaderMapExtended, not_modified_body::NotModifiedBody};
use crate::cache::CacheParams;

pub trait ResponseExtended
where
    Self: Sized,
{
    fn new_not_modified(params: &CacheParams) -> Self;
    fn json_bytes(headers: &HeaderMap, params: &CacheParams, bytes: impl FnOnce() -> Bytes)
    -> Self;
    fn static_json_bytes(headers: &HeaderMap, bytes: Bytes) -> Self;
    fn static_bytes(
        headers: &HeaderMap,
        bytes: &'static [u8],
        content_type: &'static str,
        content_encoding: &'static str,
    ) -> Self;
}

impl ResponseExtended for Response<Body> {
    fn new_not_modified(params: &CacheParams) -> Response<Body> {
        let mut response = Response::new(Body::new(NotModifiedBody));
        *response.status_mut() = StatusCode::NOT_MODIFIED;
        let headers = response.headers_mut();
        headers.insert_vary_accept_encoding();
        params.apply_to(headers);
        response
    }

    fn static_json_bytes(headers: &HeaderMap, bytes: Bytes) -> Self {
        Self::json_bytes(headers, &CacheParams::deploy(), || bytes)
    }

    fn json_bytes(
        headers: &HeaderMap,
        params: &CacheParams,
        bytes: impl FnOnce() -> Bytes,
    ) -> Self {
        if params.matches_etag(headers) {
            return Self::new_not_modified(params);
        }
        let mut response = Response::new(Body::from(bytes()));
        let h = response.headers_mut();
        h.insert_content_type_application_json();
        params.apply_to(h);
        response
    }

    fn static_bytes(
        headers: &HeaderMap,
        bytes: &'static [u8],
        content_type: &'static str,
        content_encoding: &'static str,
    ) -> Self {
        let params = CacheParams::deploy();
        if params.matches_etag(headers) {
            return Self::new_not_modified(&params);
        }
        let mut response = Response::new(Body::from(bytes));
        let h = response.headers_mut();
        h.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
        h.insert(
            header::CONTENT_ENCODING,
            HeaderValue::from_static(content_encoding),
        );
        h.insert_vary_accept_encoding();
        params.apply_to(h);
        response
    }
}
