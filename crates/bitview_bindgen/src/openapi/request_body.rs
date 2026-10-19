use crate::Ty;

#[derive(Debug, Clone)]
pub struct RequestBody {
    /// The body's type: its schema's, else a string for `text/*` and an object otherwise.
    pub(crate) ty: Ty,
    pub(crate) content_type: String,
    pub(crate) required: bool,
}
