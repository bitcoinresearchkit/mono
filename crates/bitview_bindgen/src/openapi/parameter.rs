use serde_json::Value;

/// Parameter information.
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub required: bool,
    pub param_type: String,
    pub description: Option<String>,
    /// Original OpenAPI/JSON Schema for schema-driven generators.
    pub schema: Value,
}
