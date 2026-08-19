use crate::openapi::TextSchema;

/// 200-response body shape.
#[derive(Debug, Clone)]
pub enum ResponseKind {
    /// JSON body, schema named (e.g. "Block").
    Json(String),
    /// `text/plain` body. `Some(schema)` carries a typed shape (e.g. "Height", "Hex");
    /// `None` is the escape hatch for opaque text.
    Text(Option<TextSchema>),
    /// `application/octet-stream`.
    Binary,
}

impl ResponseKind {
    /// Schema name, if the body is named (Json or typed Text).
    pub fn schema_name(&self) -> Option<&str> {
        match self {
            Self::Json(s) => Some(s.as_str()),
            Self::Text(Some(t)) => Some(t.name.as_str()),
            _ => None,
        }
    }

    /// True when a typed text body needs numeric parsing (`int(...)` etc.).
    pub fn text_is_numeric(&self) -> bool {
        matches!(self, Self::Text(Some(t)) if t.is_numeric)
    }
}
