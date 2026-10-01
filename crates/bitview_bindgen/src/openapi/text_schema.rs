/// Schema metadata for a typed `text/plain` response.
#[derive(Debug, Clone)]
pub struct TextSchema {
    /// Schema name, e.g. "Height", "Hex".
    pub(crate) name: String,
    /// True when the underlying primitive is `integer`/`number` (body needs numeric parsing).
    pub(crate) is_numeric: bool,
}
