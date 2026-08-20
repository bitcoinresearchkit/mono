use crate::openapi::{Parameter, RequestBody, ResponseKind};
use serde_json::Value;

/// Endpoint information extracted from OpenAPI spec.
#[derive(Debug, Clone)]
pub struct Endpoint {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Path template (e.g., "/blocks/{hash}")
    pub path: String,
    /// Operation ID (e.g., "getBlockByHash")
    pub operation_id: Option<String>,
    /// Short summary
    pub summary: Option<String>,
    /// Detailed description
    pub description: Option<String>,
    /// Path parameters
    pub path_params: Vec<Parameter>,
    /// Query parameters
    pub query_params: Vec<Parameter>,
    /// Request body, if any (POST/PUT/PATCH).
    pub request_body: Option<RequestBody>,
    /// Body kind for the 200 response.
    pub response_kind: ResponseKind,
    /// Raw JSON schema for the application/json 200 response, when present.
    pub json_response_schema: Option<Value>,
    /// Whether this endpoint is deprecated
    pub deprecated: bool,
    /// Whether this endpoint is explicitly excluded from MCP tool generation.
    pub mcp_ignored: bool,
    /// Whether this endpoint supports CSV format (text/csv content type)
    pub supports_csv: bool,
}

impl Endpoint {
    /// Returns true if this endpoint should be included in client generation.
    /// Non-deprecated GET and POST endpoints are included.
    pub fn should_generate(&self) -> bool {
        !self.deprecated && (self.method == "GET" || self.method == "POST")
    }

    /// Returns true if this endpoint returns JSON.
    pub fn returns_json(&self) -> bool {
        matches!(self.response_kind, ResponseKind::Json(_))
    }

    /// Returns true if this endpoint returns binary data (application/octet-stream).
    pub fn returns_binary(&self) -> bool {
        matches!(self.response_kind, ResponseKind::Binary)
    }

    /// Returns true if this endpoint returns plain text (typed or opaque).
    pub fn returns_text(&self) -> bool {
        matches!(self.response_kind, ResponseKind::Text(_))
    }

    /// Schema name attached to the response, if any.
    pub fn schema_name(&self) -> Option<&str> {
        self.response_kind.schema_name()
    }

    /// Returns the operation ID or generates one from the path.
    /// The returned string uses the raw case from the spec (typically camelCase).
    pub fn operation_name(&self) -> String {
        if let Some(op_id) = &self.operation_id {
            return op_id.clone();
        }
        let mut parts: Vec<String> = Vec::new();
        let mut prev_segment = "";

        for segment in self.path.split('/').filter(|s| !s.is_empty()) {
            if segment == "api" {
                continue;
            }
            if let Some(param) = segment.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                let prev_normalized = prev_segment.replace('-', "_");
                if !prev_normalized.ends_with(param) {
                    parts.push(format!("by_{}", param));
                }
            } else {
                let normalized = segment.replace('-', "_");
                parts.push(normalized);
                prev_segment = segment;
            }
        }
        format!("get_{}", parts.join("_"))
    }
}
