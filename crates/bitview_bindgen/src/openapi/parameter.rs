use serde_json::Value;

/// Parameter information.
#[derive(Debug, Clone)]
pub struct Parameter {
    pub(crate) name: String,
    pub(crate) required: bool,
    pub(crate) param_type: String,
    pub(crate) description: Option<String>,
    /// Original OpenAPI/JSON Schema for schema-driven generators.
    pub(crate) schema: Value,
}
