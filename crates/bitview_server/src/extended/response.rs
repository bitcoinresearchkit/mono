use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, Response, StatusCode, header},
    response::IntoResponse,
};

use super::header_map::HeaderMapExtended;
use crate::cache::CacheParams;

pub trait ResponseExtended
where
    Self: Sized,
{
    fn new_not_modified(params: &CacheParams) -> Self;
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
        let mut response = (StatusCode::NOT_MODIFIED, "").into_response();
        params.apply_to(response.headers_mut());
        response
    }

    fn static_json_bytes(headers: &HeaderMap, bytes: Bytes) -> Self {
        let params = CacheParams::deploy();
        if params.matches_etag(headers) {
            return Self::new_not_modified(&params);
        }
        let mut response = Response::new(Body::from(bytes));
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
        h.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
        h.insert(header::CONTENT_ENCODING, content_encoding.parse().unwrap());
        h.insert_vary_accept_encoding();
        params.apply_to(h);
        response
    }
}
