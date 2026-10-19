use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};

#[derive(Debug)]
pub struct Error(StatusCode, String);

impl Error {
    pub(crate) fn not_found(msg: impl Into<String>) -> Self {
        Self(StatusCode::NOT_FOUND, msg.into())
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
