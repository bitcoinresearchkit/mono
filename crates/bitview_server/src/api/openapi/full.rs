use std::sync::Arc;

use aide::openapi::OpenApi;
use axum::{body::Bytes, http::HeaderMap, response::Response};
use serde_json::to_vec;

use crate::prepared_json::PreparedJson;

/// Full OpenAPI spec, pre-serialized at startup with its content validator.
#[derive(Clone)]
pub struct OpenApiJson(Arc<PreparedJson>);

impl OpenApiJson {
    pub fn new(openapi: &OpenApi) -> Self {
        Self(Arc::new(PreparedJson::from_bytes(Bytes::from(
            to_vec(openapi).unwrap(),
        ))))
    }

    pub fn respond(&self, headers: &HeaderMap) -> Response {
        self.0.respond(headers)
    }
}
