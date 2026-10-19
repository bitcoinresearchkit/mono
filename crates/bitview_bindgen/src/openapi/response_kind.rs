use crate::Ty;

/// 200-response body shape.
#[derive(Debug, Clone)]
pub enum ResponseKind {
    Json(Ty),
    /// `text/plain` body: a named type (`Height`, `Txid`, `Hex`) or plain text (`Ty::String`).
    /// `numeric` when the body is a number (`Height`), parsed as one.
    Text {
        ty: Ty,
        numeric: bool,
    },
    /// `application/octet-stream`.
    Binary,
}

impl ResponseKind {
    /// The body's type; none for binary.
    pub(crate) fn ty(&self) -> Option<&Ty> {
        match self {
            Self::Json(ty) | Self::Text { ty, .. } => Some(ty),
            Self::Binary => None,
        }
    }

    pub(crate) fn text_is_numeric(&self) -> bool {
        matches!(self, Self::Text { numeric: true, .. })
    }
}
