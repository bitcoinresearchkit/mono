use crate::openapi::{Parameter, RequestBody, ResponseKind};
use serde_json::Value;

/// Endpoint information extracted from OpenAPI spec.
#[derive(Debug, Clone)]
pub struct Endpoint {
    /// HTTP method (GET, POST, etc.)
    pub(crate) method: String,
    /// Path template (e.g., "/blocks/{hash}")
    pub(crate) path: String,
    /// Operation ID (e.g., "get_block"), naming every generated method, command and tool
    pub(crate) operation_id: String,
    /// Short summary
    pub(crate) summary: Option<String>,
    /// Detailed description
    pub(crate) description: Option<String>,
    /// Path parameters
    pub(crate) path_params: Vec<Parameter>,
    /// Query parameters
    pub(crate) query_params: Vec<Parameter>,
    /// Request body, if any (POST/PUT/PATCH).
    pub(crate) request_body: Option<RequestBody>,
    /// Body kind for the 200 response.
    pub(crate) response_kind: ResponseKind,
    /// Raw JSON schema for the application/json 200 response, when present.
    pub(crate) json_response_schema: Option<Value>,
    /// Whether this endpoint is deprecated
    pub(crate) deprecated: bool,
    /// Whether this endpoint is explicitly excluded from MCP tool generation.
    pub(crate) mcp_ignored: bool,
    /// Whether this endpoint supports CSV format (text/csv content type)
    pub(crate) supports_csv: bool,
}

impl Endpoint {
    /// Returns true if this endpoint should be included in client generation.
    /// Non-deprecated GET and POST endpoints are included.
    pub(crate) fn should_generate(&self) -> bool {
        !self.deprecated && (self.method == "GET" || self.method == "POST")
    }

    /// Returns true if this endpoint returns JSON.
    pub(crate) fn returns_json(&self) -> bool {
        matches!(self.response_kind, ResponseKind::Json(_))
    }

    /// Returns true if this endpoint returns binary data (application/octet-stream).
    pub(crate) fn returns_binary(&self) -> bool {
        matches!(self.response_kind, ResponseKind::Binary)
    }

    /// Schema name attached to the response, if any.
    pub(crate) fn schema_name(&self) -> Option<&str> {
        self.response_kind.schema_name()
    }
}
