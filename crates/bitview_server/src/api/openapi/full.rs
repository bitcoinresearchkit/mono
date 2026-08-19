use aide::openapi::OpenApi;
use axum::body::Bytes;

/// Full OpenAPI spec, pre-serialized at startup and served as raw bytes per request.
#[derive(Clone)]
pub struct OpenApiJson(Bytes);

impl OpenApiJson {
    pub fn new(openapi: &OpenApi) -> Self {
        Self(Bytes::from(serde_json::to_vec(openapi).unwrap()))
    }

    pub fn bytes(&self) -> Bytes {
        self.0.clone()
    }
}
