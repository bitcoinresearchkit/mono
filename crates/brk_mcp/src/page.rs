use axum::{
    http::{HeaderValue, header::CACHE_CONTROL},
    response::{Html, IntoResponse, Response},
};

const HTML: &str = include_str!("../assets/index.html");

pub async fn get() -> Response {
    let mut response = Html(HTML).into_response();
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=300"),
    );
    response
}
