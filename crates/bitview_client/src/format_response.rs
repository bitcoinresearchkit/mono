/// The answer of an endpoint with a `format` parameter: CSV text for `Format::CSV`, else JSON.
#[derive(Debug, Clone)]
pub enum FormatResponse<T> {
    Json(T),
    Csv(String),
}
