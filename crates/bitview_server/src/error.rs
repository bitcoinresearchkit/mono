use std::result::Result as StdResult;

use aide::OperationOutput;
use axum::{
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use brk_error::Error as BrkError;
use serde_json::to_vec;

use crate::{
    cache::{CacheParams, ErrorCachePolicy},
    error_body::ErrorBody,
    error_code::ErrorCode,
};

const DOC_URL: &str = "/api";

pub type Result<T> = StdResult<T, Error>;

fn error_type(status: StatusCode) -> &'static str {
    match status {
        StatusCode::BAD_REQUEST => "invalid_request",
        StatusCode::FORBIDDEN => "forbidden",
        StatusCode::NOT_FOUND => "not_found",
        StatusCode::SERVICE_UNAVAILABLE => "unavailable",
        StatusCode::GATEWAY_TIMEOUT => "timeout",
        _ => "internal",
    }
}

fn error_details(error: &BrkError) -> (StatusCode, ErrorCode) {
    match error {
        BrkError::InvalidAddr => (StatusCode::BAD_REQUEST, ErrorCode::InvalidAddr),
        BrkError::InvalidTxid => (StatusCode::BAD_REQUEST, ErrorCode::InvalidTxid),
        BrkError::InvalidNetwork => (StatusCode::BAD_REQUEST, ErrorCode::InvalidNetwork),
        BrkError::UnsupportedType(_) => (StatusCode::BAD_REQUEST, ErrorCode::UnsupportedType),
        BrkError::Parse(_) => (StatusCode::BAD_REQUEST, ErrorCode::ParseError),
        BrkError::NoSeries => (StatusCode::BAD_REQUEST, ErrorCode::NoSeries),
        BrkError::SeriesUnsupportedIndex { .. } => {
            (StatusCode::BAD_REQUEST, ErrorCode::SeriesUnsupportedIndex)
        }
        BrkError::WeightExceeded { .. } => (StatusCode::BAD_REQUEST, ErrorCode::WeightExceeded),
        BrkError::TooManyUtxos => (StatusCode::BAD_REQUEST, ErrorCode::TooManyUtxos),
        BrkError::UnknownAddr => (StatusCode::NOT_FOUND, ErrorCode::UnknownAddr),
        BrkError::UnknownTxid => (StatusCode::NOT_FOUND, ErrorCode::UnknownTxid),
        BrkError::NotFound(_) => (StatusCode::NOT_FOUND, ErrorCode::NotFound),
        BrkError::OutOfRange(_) => (StatusCode::NOT_FOUND, ErrorCode::OutOfRange),
        BrkError::UnindexableDate => (StatusCode::NOT_FOUND, ErrorCode::UnindexableDate),
        BrkError::NoData => (StatusCode::NOT_FOUND, ErrorCode::NoData),
        BrkError::SeriesNotFound(_) => (StatusCode::NOT_FOUND, ErrorCode::SeriesNotFound),
        BrkError::MempoolNotAvailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            ErrorCode::MempoolNotAvailable,
        ),
        BrkError::ReadTimeout => (StatusCode::GATEWAY_TIMEOUT, ErrorCode::Timeout),
        BrkError::StateUpdating => (StatusCode::SERVICE_UNAVAILABLE, ErrorCode::StateUpdating),
        BrkError::AuthFailed => (StatusCode::FORBIDDEN, ErrorCode::AuthFailed),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalError),
    }
}

/// Server error type that maps to HTTP status codes and structured JSON.
pub struct Error {
    status: StatusCode,
    code: ErrorCode,
    message: String,
}

impl Error {
    pub fn new(status: StatusCode, code: ErrorCode, msg: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: msg.into(),
        }
    }

    pub fn timeout(action: bool) -> Self {
        Self::new(
            StatusCode::GATEWAY_TIMEOUT,
            ErrorCode::Timeout,
            if action {
                "Request timed out; submission outcome may be unknown"
            } else {
                "Request timed out waiting for available data or capacity"
            },
        )
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ErrorCode::BadRequest, msg)
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorCode::NotFound, msg)
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            msg,
        )
    }

    #[cfg(any(feature = "chain", test))]
    pub fn overloaded(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, ErrorCode::Overloaded, msg)
    }

    fn cache_policy(&self) -> ErrorCachePolicy {
        match self.code {
            ErrorCode::InvalidAddr | ErrorCode::InvalidNetwork | ErrorCode::InvalidTxid => {
                ErrorCachePolicy::Immutable
            }
            _ => match self.status {
                StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => ErrorCachePolicy::Revalidate,
                _ => ErrorCachePolicy::NoStore,
            },
        }
    }
}

impl From<BrkError> for Error {
    fn from(e: BrkError) -> Self {
        let (status, code) = error_details(&e);
        Self {
            status,
            code,
            message: e.to_string(),
        }
    }
}

impl OperationOutput for Error {
    type Inner = ();
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let policy = self.cache_policy();
        let body = to_vec(&ErrorBody::new(
            error_type(self.status),
            self.code.as_str(),
            self.message,
            DOC_URL,
        ))
        .unwrap();
        let mut response = (
            self.status,
            [(header::CONTENT_TYPE, "application/problem+json")],
            body,
        )
            .into_response();
        if self.code.is_transient() {
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
        }
        CacheParams::apply_error_cache_control(response.headers_mut(), policy);
        response
    }
}
