use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, HeaderValue, Response, StatusCode, header},
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
        let mut response = Response::new(Body::empty());
        *response.status_mut() = StatusCode::NOT_MODIFIED;
        let headers = response.headers_mut();
        headers.insert_vary_accept_encoding();
        params.apply_to(headers);
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

#[cfg(test)]
mod tests {
    use axum::{
        body::to_bytes,
        http::{
            HeaderName,
            header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, ETAG, VARY},
        },
    };

    use super::*;

    #[tokio::test]
    async fn not_modified_is_empty_and_preserves_cache_metadata() {
        let response = Response::new_not_modified(&CacheParams::deploy());

        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
        assert!(response.headers().contains_key(ETAG));
        assert!(response.headers().contains_key(CACHE_CONTROL));
        assert!(
            response
                .headers()
                .contains_key(HeaderName::from_static("cdn-cache-control"))
        );
        assert_eq!(
            response.headers().get(VARY),
            Some(&HeaderValue::from_static("Accept-Encoding"))
        );
        assert!(!response.headers().contains_key(CONTENT_TYPE));
        assert!(!response.headers().contains_key(CONTENT_LENGTH));
        assert!(
            to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .is_empty()
        );
    }
}
