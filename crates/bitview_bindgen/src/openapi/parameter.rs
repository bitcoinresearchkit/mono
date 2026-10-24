use serde_json::Value;

use crate::Ty;

#[derive(Debug, Clone)]
pub struct Parameter {
    /// The wire name, as requests send it (`txId[]`).
    pub(crate) name: String,
    /// The name client code gives it: the server's `x-arg-name` (`txids`), else the wire name
    /// without brackets.
    pub(crate) arg: String,
    pub(crate) required: bool,
    pub(crate) ty: Ty,
    pub(crate) description: Option<String>,
    /// Original OpenAPI/JSON Schema for schema-driven generators.
    pub(crate) schema: Value,
}
