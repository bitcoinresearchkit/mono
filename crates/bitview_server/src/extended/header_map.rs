use axum::http::{HeaderMap, HeaderName, HeaderValue, header};

pub trait HeaderMapExtended {
    fn insert_cache_control(&mut self, value: &'static str);
    fn insert_cdn_cache_control(&mut self, value: &'static str);

    #[cfg(feature = "series")]
    fn insert_content_disposition_attachment(&mut self, filename: &str);

    fn insert_content_type_application_json(&mut self);
    #[cfg(feature = "series")]
    fn insert_content_type_text_csv(&mut self);

    fn insert_vary_accept_encoding(&mut self);

    #[cfg(all(feature = "series", feature = "urpd"))]
    fn insert_deprecation(&mut self, sunset: &'static str);
}

impl HeaderMapExtended for HeaderMap {
    fn insert_cache_control(&mut self, value: &'static str) {
        self.insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
    }

    fn insert_cdn_cache_control(&mut self, value: &'static str) {
        self.insert(
            HeaderName::from_static("cdn-cache-control"),
            HeaderValue::from_static(value),
        );
    }

    #[cfg(feature = "series")]
    fn insert_content_disposition_attachment(&mut self, filename: &str) {
        self.insert(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\"")
                .parse()
                .unwrap(),
        );
    }

    fn insert_content_type_application_json(&mut self) {
        self.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
    }

    #[cfg(feature = "series")]
    fn insert_content_type_text_csv(&mut self) {
        self.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/csv"));
    }

    fn insert_vary_accept_encoding(&mut self) {
        self.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
    }

    #[cfg(all(feature = "series", feature = "urpd"))]
    fn insert_deprecation(&mut self, sunset: &'static str) {
        self.insert("Deprecation", HeaderValue::from_static("true"));
        self.insert("Sunset", HeaderValue::from_static(sunset));
    }
}
