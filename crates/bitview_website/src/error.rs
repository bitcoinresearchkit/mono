use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};

/// Website error type that maps to HTTP status codes.
#[derive(Debug)]
pub struct Error(StatusCode, String);

impl Error {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self(StatusCode::NOT_FOUND, msg.into())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response<Body> {
        (self.0, self.1).into_response()
    }
}
