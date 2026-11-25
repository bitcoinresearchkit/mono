use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{handlers::file_handler, page_info::Pages, website};

pub(crate) struct Site {
    pub(crate) pages: Pages,
    pub(crate) cdn_purged: bool,
}

/// The site's routes. `cdn_purged` says a CDN in front is purged on every release, so it may keep until then what
/// only a release changes.
pub fn router(cdn_purged: bool) -> Router {
    website::log();
    // Validate all embedded pages at startup, including sites without page info.
    let _ = website::release();
    let pages = Pages::load().unwrap_or_else(|error| panic!("Website page info: {error}"));
    Router::new()
        .route("/{*path}", get(file_handler))
        .route("/", get(file_handler))
        .with_state(Arc::new(Site { pages, cdn_purged }))
}
