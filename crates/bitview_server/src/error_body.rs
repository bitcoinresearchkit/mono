use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct ErrorBody {
    error: ErrorDetail,
}

impl ErrorBody {
    pub fn new(
        r#type: &'static str,
        code: &'static str,
        message: String,
        doc_url: &'static str,
    ) -> Self {
        Self {
            error: ErrorDetail {
                r#type,
                code,
                message,
                doc_url,
            },
        }
    }
}

#[derive(Serialize, JsonSchema)]
struct ErrorDetail {
    /// Error category: "invalid_request", "forbidden", "not_found", "unavailable", or "internal"
    #[schemars(with = "String")]
    r#type: &'static str,
    /// Machine-readable error code (e.g. "invalid_addr", "series_not_found")
    #[schemars(with = "String")]
    code: &'static str,
    /// Human-readable description
    message: String,
    /// Link to API documentation
    #[schemars(with = "String")]
    doc_url: &'static str,
}
