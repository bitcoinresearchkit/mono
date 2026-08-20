/// Response type for endpoints that support multiple formats (JSON/CSV).
#[derive(Debug, Clone)]
pub enum FormatResponse<T> {
    /// JSON response, deserialized to T.
    Json(T),
    /// CSV response as raw string.
    Csv(String),
}

impl<T> FormatResponse<T> {
    /// Unwrap the JSON variant, panicking if this is CSV.
    pub fn json(self) -> T {
        match self {
            FormatResponse::Json(v) => v,
            FormatResponse::Csv(_) => panic!("expected JSON response, got CSV"),
        }
    }

    /// Unwrap the CSV variant, panicking if this is JSON.
    pub fn csv(self) -> String {
        match self {
            FormatResponse::Csv(s) => s,
            FormatResponse::Json(_) => panic!("expected CSV response, got JSON"),
        }
    }

    /// Returns true if this is a JSON response.
    pub fn is_json(&self) -> bool {
        matches!(self, FormatResponse::Json(_))
    }

    /// Returns true if this is a CSV response.
    pub fn is_csv(&self) -> bool {
        matches!(self, FormatResponse::Csv(_))
    }
}
