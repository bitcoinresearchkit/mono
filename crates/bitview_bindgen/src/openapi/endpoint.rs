use crate::{Ty, openapi::{Parameter, RequestBody, ResponseKind}};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Endpoint {
    /// Uppercase (`GET`, `POST`).
    pub(crate) method: String,
    /// Path template (e.g., "/blocks/{hash}")
    pub(crate) path: String,
    /// Operation ID (e.g., "get_block"), naming every generated method, command and tool
    pub(crate) operation_id: String,
    pub(crate) summary: Option<String>,
    pub(crate) description: Option<String>,
    /// In path order.
    pub(crate) path_params: Vec<Parameter>,
    pub(crate) query_params: Vec<Parameter>,
    pub(crate) request_body: Option<RequestBody>,
    /// Body kind for the 200 response.
    pub(crate) response_kind: ResponseKind,
    /// An explicitly declared empty 204, in addition to the JSON 200 response.
    pub(crate) no_content: bool,
    /// Client result schema, including null for a declared 204, when present.
    pub(crate) json_response_schema: Option<Value>,
    pub(crate) deprecated: bool,
    /// `x-mcp-ignore: true`: no MCP tool.
    pub(crate) mcp_ignored: bool,
    /// The 200 response offers `text/csv`.
    pub(crate) supports_csv: bool,
}

impl Endpoint {
    /// Non-deprecated `GET` and `POST` operations get client methods.
    pub(crate) fn should_generate(&self) -> bool {
        !self.deprecated && (self.method == "GET" || self.method == "POST")
    }

    /// The client result, distinct from the body type used to decode a present answer.
    pub(crate) fn response_ty(&self) -> Option<Ty> {
        self.response_kind.ty().cloned().map(|ty| {
            if self.no_content { ty.nullable() } else { ty }
        })
    }

    pub(crate) fn returns_json(&self) -> bool {
        matches!(self.response_kind, ResponseKind::Json(_))
    }

    pub(crate) fn returns_binary(&self) -> bool {
        matches!(self.response_kind, ResponseKind::Binary)
    }
}
