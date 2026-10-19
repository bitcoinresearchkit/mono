use axum::{
    body::Body,
    http::{HeaderValue, Uri, header::CACHE_CONTROL},
    response::{IntoResponse, Response},
};

const PNG: &[u8] = include_bytes!("../assets/logo.png");

const VERSION_QUERY: &str = concat!("v=", env!("CARGO_PKG_VERSION"));

/// The logo's URL path: versioned, so its year-long cache never outlives a release.
pub const PATH: &str = concat!("logo.png?v=", env!("CARGO_PKG_VERSION"));

/// The year-long cache only under this release's versioned URL; any other revalidates.
pub async fn get(uri: Uri) -> Response {
    let mut response = Body::from(PNG).into_response();
    response
        .headers_mut()
        .insert("content-type", HeaderValue::from_static("image/png"));
    let cache_control = if uri.query() == Some(VERSION_QUERY) {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=1, must-revalidate"
    };
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(cache_control));
    response.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    response
}
