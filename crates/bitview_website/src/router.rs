use axum::{Router, routing::get};

use crate::{
    Website,
    handlers::{file_handler, index_handler},
};

/// Create a router for serving the website.
///
/// Returns an empty router if the website is disabled.
pub fn router(website: Website) -> Router {
    if website.is_enabled() {
        Router::new()
            .route("/{*path}", get(file_handler))
            .route("/", get(index_handler))
            .with_state(website)
    } else {
        Router::new()
    }
}
