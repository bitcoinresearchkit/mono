use std::path::Path;

use axum::http::{HeaderMap, header};

pub trait HeaderMapExtended {
    fn has_etag(&self, etag: &str) -> bool;
    fn insert_etag(&mut self, etag: &str);
    fn insert_cache_control_must_revalidate(&mut self);
    fn insert_cache_control_immutable(&mut self);
    fn insert_content_type(&mut self, path: &Path);
    fn insert_content_type_text_html(&mut self);
}

impl HeaderMapExtended for HeaderMap {
    fn has_etag(&self, etag: &str) -> bool {
        self.get(header::IF_NONE_MATCH).is_some_and(|v| v == etag)
    }

    fn insert_etag(&mut self, etag: &str) {
        self.insert(header::ETAG, etag.parse().unwrap());
    }

    fn insert_cache_control_must_revalidate(&mut self) {
        self.insert(
            header::CACHE_CONTROL,
            "public, max-age=1, must-revalidate".parse().unwrap(),
        );
    }

    fn insert_cache_control_immutable(&mut self) {
        self.insert(
            header::CACHE_CONTROL,
            "public, max-age=31536000, immutable".parse().unwrap(),
        );
    }

    fn insert_content_type(&mut self, path: &Path) {
        let content_type = match path
            .extension()
            .map(|s| s.to_str().unwrap_or_default())
            .unwrap_or_default()
        {
            "js" | "mjs" => "application/javascript",
            "json" | "map" => "application/json",
            "html" => "text/html",
            "css" => "text/css",
            "toml" | "txt" => "text/plain",
            "pdf" => "application/pdf",
            "woff2" => "font/woff2",
            "ico" => "image/x-icon",
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "svg" => "image/svg+xml",
            "webmanifest" => "application/manifest+json",
            _ => return,
        };
        self.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    }

    fn insert_content_type_text_html(&mut self) {
        self.insert(header::CONTENT_TYPE, "text/html".parse().unwrap());
    }
}
