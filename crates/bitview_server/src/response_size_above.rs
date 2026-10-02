use axum::{
    body::HttpBody,
    http::{Response, StatusCode, header::CONTENT_LENGTH},
};
use tower_http::compression::predicate::Predicate;

/// Prefer representation metadata to the empty body Axum supplies for HEAD.
#[derive(Clone, Copy)]
pub struct ResponseSizeAbove(pub u64);

impl Predicate for ResponseSizeAbove {
    fn should_compress<B: HttpBody>(&self, response: &Response<B>) -> bool {
        // 304 deliberately has no length hint: it describes a representation,
        // not an unknown-length stream that needs an encoder.
        if response.status() == StatusCode::NOT_MODIFIED {
            return false;
        }
        let body_size = response.body().size_hint().exact();
        let size = body_size
            .filter(|&size| size != 0)
            .or_else(|| {
                response
                    .headers()
                    .get(CONTENT_LENGTH)
                    .and_then(|value| value.to_str().ok()?.parse::<u64>().ok())
            })
            .or(body_size);
        size.is_none_or(|size| size >= self.0)
    }
}
