/// Request body shape for POST/PUT/PATCH endpoints.
#[derive(Debug, Clone)]
pub struct RequestBody {
    /// Body content type as a name (e.g. "string" for text/plain, "Foo" for an `application/json` $ref).
    pub body_type: String,
    /// Media type selected from the OpenAPI request body content map.
    pub content_type: String,
    /// Whether the body is required.
    pub required: bool,
}
