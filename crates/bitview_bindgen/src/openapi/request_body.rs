use crate::Ty;

/// Request body shape for POST/PUT/PATCH endpoints.
#[derive(Debug, Clone)]
pub struct RequestBody {
    /// The body's type: its schema's, else a string for `text/*` and an object otherwise.
    pub(crate) ty: Ty,
    /// Media type selected from the OpenAPI request body content map.
    pub(crate) content_type: String,
    /// Whether the body is required.
    pub(crate) required: bool,
}
