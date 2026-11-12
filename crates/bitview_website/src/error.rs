use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};

#[derive(Debug)]
pub struct Error(StatusCode, String);

impl Error {
    /// Nothing at the address.
    pub(crate) fn not_found() -> Self {
        Self(StatusCode::NOT_FOUND, "File not found".into())
    }

    pub(crate) fn is_not_found(&self) -> bool {
        self.0 == StatusCode::NOT_FOUND
    }

    pub(crate) fn internal(msg: impl Into<String>) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, msg.into())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response<Body> {
        (self.0, self.1).into_response()
    }
}
