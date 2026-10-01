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
    pub(crate) operation_id: Option<String>,
    /// Short summary
    pub(crate) summary: Option<String>,
    /// Detailed description
    pub(crate) description: Option<String>,
    /// Path parameters
    pub(crate) path_params: Vec<Parameter>,
    /// Query parameters
    pub query_params: Vec<Parameter>,
    /// Request body, if any (POST/PUT/PATCH).
    pub request_body: Option<RequestBody>,
    /// Body kind for the 200 response.
    pub(crate) response_kind: ResponseKind,
    /// Raw JSON schema for the application/json 200 response, when present.
    pub(crate) json_response_schema: Option<Value>,
    /// Whether this endpoint is deprecated
    pub(crate) deprecated: bool,
    /// Whether this endpoint is explicitly excluded from MCP tool generation.
    pub(crate) mcp_ignored: bool,
    /// Whether this endpoint supports CSV format (text/csv content type)
    pub supports_csv: bool,
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

    /// Returns true if this endpoint returns plain text (typed or opaque).
    pub(crate) fn returns_text(&self) -> bool {
        matches!(self.response_kind, ResponseKind::Text(_))
    }

    /// Schema name attached to the response, if any.
    pub(crate) fn schema_name(&self) -> Option<&str> {
        self.response_kind.schema_name()
    }

    /// Returns the operation ID or generates one from the path.
    /// The returned string uses the raw case from the spec (typically camelCase).
    pub(crate) fn operation_name(&self) -> String {
        if let Some(op_id) = &self.operation_id {
            return op_id.clone();
        }
        let mut result = String::from("get");
        let mut previous = String::new();

        for segment in self.path.split('/').filter(|s| !s.is_empty()) {
            if segment == "api" {
                continue;
            }
            if let Some(param) = segment.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                if !previous.ends_with(param) {
                    result.push_str("_by_");
                    result.push_str(param);
                }
            } else {
                previous = segment.replace('-', "_");
                result.push('_');
                result.push_str(&previous);
            }
        }
        if result == "get" {
            result.push('_');
        }
        result
    }
}

#[cfg(test)]
#[path = "../../tests/unit/openapi_endpoint.rs"]
mod tests;
