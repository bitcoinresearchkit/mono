use std::fmt::Write;

use axum::http::{HeaderMap, HeaderValue, header};

pub trait HeaderMapExtended {
    fn insert_content_disposition_attachment(&mut self, filename: &str);
    fn insert_content_type_application_json(&mut self);
    fn insert_content_type_text_csv(&mut self);
}

impl HeaderMapExtended for HeaderMap {
    fn insert_content_disposition_attachment(&mut self, filename: &str) {
        if filename.is_ascii() && !filename.contains('"') && !filename.contains('\\') {
            let mut value = String::with_capacity(filename.len() + 23);
            value.push_str("attachment; filename=\"");
            value.push_str(filename);
            value.push('"');
            if let Ok(value) = HeaderValue::try_from(value) {
                self.insert(header::CONTENT_DISPOSITION, value);
                return;
            }
        }
        let mut value = String::from("attachment; filename*=UTF-8''");
        for byte in filename.bytes() {
            write!(value, "%{byte:02X}").unwrap();
        }
        self.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::try_from(value).expect("ASCII attachment header"),
        );
    }

    fn insert_content_type_application_json(&mut self) {
        self.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
    }

    fn insert_content_type_text_csv(&mut self) {
        self.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/csv"));
    }
}
