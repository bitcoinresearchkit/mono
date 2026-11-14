use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{Website, handlers::file_handler, page_info::Pages};

pub(crate) struct Site {
    pub(crate) website: Website,
    pub(crate) pages: Pages,
}

/// Returns an empty router if the website is disabled.
pub fn router(website: Website) -> Router {
    if website.is_enabled() {
        let pages =
            Pages::load(&website).unwrap_or_else(|error| panic!("Website page info: {error}"));
        Router::new()
            .route("/{*path}", get(file_handler))
            .route("/", get(file_handler))
            .with_state(Arc::new(Site { website, pages }))
    } else {
        Router::new()
    }
}
