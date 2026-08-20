use axum::{
    body::Body,
    http::{HeaderValue, header::CACHE_CONTROL},
    response::{IntoResponse, Response},
};

const PNG: &[u8] = include_bytes!("../assets/logo.png");

pub async fn get() -> Response {
    let mut response = Body::from(PNG).into_response();
    response
        .headers_mut()
        .insert("content-type", HeaderValue::from_static("image/png"));
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    response.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    response
}
