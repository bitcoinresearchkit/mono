/// Schema metadata for a typed `text/plain` response.
#[derive(Debug, Clone)]
pub struct TextSchema {
    /// Schema name, e.g. "Height", "Hex".
    pub name: String,
    /// True when the underlying primitive is `integer`/`number` (body needs numeric parsing).
    pub is_numeric: bool,
}
