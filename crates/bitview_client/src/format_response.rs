/// Response type for endpoints that support multiple formats (JSON/CSV).
#[derive(Debug, Clone)]
pub enum FormatResponse<T> {
    /// JSON response, deserialized to T.
    Json(T),
    /// CSV response as raw string.
    Csv(String),
}
