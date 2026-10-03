use std::result::Result as StdResult;

use aide::OperationOutput;
use axum::{
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use bitview_query::Error as QueryError;
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

/// Every query error has a deliberate status: a new variant does not compile until it gets one.
fn error_details(error: &QueryError) -> (StatusCode, ErrorCode) {
    match error {
        QueryError::InvalidAddr => (StatusCode::BAD_REQUEST, ErrorCode::InvalidAddr),
        QueryError::InvalidNetwork => (StatusCode::BAD_REQUEST, ErrorCode::InvalidNetwork),
        QueryError::InvalidParam(_) => (StatusCode::BAD_REQUEST, ErrorCode::ParseError),
        QueryError::UnsupportedType(_) => (StatusCode::BAD_REQUEST, ErrorCode::UnsupportedType),
        QueryError::NoSeries => (StatusCode::BAD_REQUEST, ErrorCode::NoSeries),
        QueryError::SeriesUnsupportedIndex { .. } => {
            (StatusCode::BAD_REQUEST, ErrorCode::SeriesUnsupportedIndex)
        }
        QueryError::WeightExceeded { .. } => (StatusCode::BAD_REQUEST, ErrorCode::WeightExceeded),
        QueryError::TooManyUtxos => (StatusCode::BAD_REQUEST, ErrorCode::TooManyUtxos),
        QueryError::UnknownAddr => (StatusCode::NOT_FOUND, ErrorCode::UnknownAddr),
        QueryError::UnknownTxid => (StatusCode::NOT_FOUND, ErrorCode::UnknownTxid),
        QueryError::NotFound(_) => (StatusCode::NOT_FOUND, ErrorCode::NotFound),
        QueryError::OutOfRange(_) => (StatusCode::NOT_FOUND, ErrorCode::OutOfRange),
        QueryError::UnindexableDate => (StatusCode::NOT_FOUND, ErrorCode::UnindexableDate),
        QueryError::NoData => (StatusCode::NOT_FOUND, ErrorCode::NoData),
        QueryError::SeriesNotFound(_) => (StatusCode::NOT_FOUND, ErrorCode::SeriesNotFound),
        QueryError::MempoolNotAvailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            ErrorCode::MempoolNotAvailable,
        ),
        QueryError::StateUpdating => (StatusCode::SERVICE_UNAVAILABLE, ErrorCode::StateUpdating),
        QueryError::ReadTimeout => (StatusCode::GATEWAY_TIMEOUT, ErrorCode::Timeout),
        QueryError::Internal(_) | QueryError::Lower(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalError)
        }
    }
}

/// Server error type that maps to HTTP status codes and structured JSON.
pub struct Error {
    status: StatusCode,
    code: ErrorCode,
    message: String,
}

impl Error {
    pub(crate) fn new(status: StatusCode, code: ErrorCode, msg: impl Into<String>) -> Self {
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

    pub(crate) fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ErrorCode::BadRequest, msg)
    }

    pub(crate) fn not_found(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorCode::NotFound, msg)
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            msg,
        )
    }

    #[cfg(feature = "chain")]
    pub fn overloaded(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, ErrorCode::Overloaded, msg)
    }

    fn cache_policy(&self) -> ErrorCachePolicy {
        match self.code {
            ErrorCode::InvalidAddr | ErrorCode::InvalidNetwork => ErrorCachePolicy::Immutable,
            _ => match self.status {
                StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => ErrorCachePolicy::Revalidate,
                _ => ErrorCachePolicy::NoStore,
            },
        }
    }
}

impl From<QueryError> for Error {
    fn from(error: QueryError) -> Self {
        let (status, code) = error_details(&error);
        Self {
            status,
            code,
            message: error.to_string(),
        }
    }
}

/// Errors from below the query (node calls, IO) take the query's translation, except a
/// transaction the node refused, which is the client's input.
impl From<BrkError> for Error {
    fn from(error: BrkError) -> Self {
        match error {
            BrkError::TxRejected(reason) => {
                Self::new(StatusCode::BAD_REQUEST, ErrorCode::ParseError, reason)
            }
            error => QueryError::from(error).into(),
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
