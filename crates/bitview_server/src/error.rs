use std::result::Result as StdResult;

use aide::OperationOutput;
use axum::{
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use bitview_http::CachePolicy;
use bitview_query::{Error as QueryError, SeriesNotFound};
use bitview_types::{ErrorBody, ErrorCode, ErrorType};
use brk_error::Error as BrkError;
use serde_json::to_vec;
use tracing::error;

const DOC_URL: &str = "/api";
/// The website's page of what each release renamed or removed (series, endpoints, indexes, parameters).
const CHANGES_URL: &str = "/changes/";

/// What an answer to a request written for an earlier release adds, after its own sentence: an id's own answer on the
/// changes page, else the page.
pub(crate) fn changes_hint(id: Option<&str>) -> String {
    match id {
        Some(id) => format!("What became of it: {CHANGES_URL}?id={id}"),
        None => format!("Written for an earlier release? What it renamed or removed: {CHANGES_URL}"),
    }
}

pub type Result<T> = StdResult<T, Error>;

fn error_type(status: StatusCode) -> ErrorType {
    match status {
        StatusCode::NOT_FOUND => ErrorType::NotFound,
        StatusCode::SERVICE_UNAVAILABLE => ErrorType::Unavailable,
        StatusCode::GATEWAY_TIMEOUT => ErrorType::Timeout,
        status if status.is_client_error() => ErrorType::InvalidRequest,
        _ => ErrorType::Internal,
    }
}

/// Every query error has a deliberate status: a new variant does not compile until it gets one.
fn error_details(error: &QueryError) -> (StatusCode, ErrorCode) {
    match error {
        QueryError::InvalidAddress => (StatusCode::BAD_REQUEST, ErrorCode::InvalidAddress),
        QueryError::InvalidNetwork => (StatusCode::BAD_REQUEST, ErrorCode::InvalidNetwork),
        QueryError::InvalidParam(_) => (StatusCode::BAD_REQUEST, ErrorCode::BadRequest),
        QueryError::UnsupportedType(_) => (StatusCode::BAD_REQUEST, ErrorCode::UnsupportedType),
        QueryError::NoSeries => (StatusCode::BAD_REQUEST, ErrorCode::NoSeries),
        QueryError::SeriesUnsupportedIndex { .. } => {
            (StatusCode::BAD_REQUEST, ErrorCode::SeriesUnsupportedIndex)
        }
        QueryError::WeightExceeded { .. } => (StatusCode::BAD_REQUEST, ErrorCode::WeightExceeded),
        QueryError::TooManyUtxos { .. } => (StatusCode::BAD_REQUEST, ErrorCode::TooManyUtxos),
        QueryError::LimitExceeded(_) => (StatusCode::BAD_REQUEST, ErrorCode::LimitExceeded),
        QueryError::UnknownAfterTxid => (
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::UnknownAfterTxid,
        ),
        QueryError::UnknownAddress => (StatusCode::NOT_FOUND, ErrorCode::UnknownAddress),
        QueryError::UnknownTxid => (StatusCode::NOT_FOUND, ErrorCode::UnknownTxid),
        QueryError::UnknownPool => (StatusCode::NOT_FOUND, ErrorCode::UnknownPool),
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

    #[cfg(feature = "full-api")]
    pub fn overloaded(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, ErrorCode::Overloaded, msg)
    }

    fn cache_policy(&self) -> CachePolicy {
        match self.status {
            StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => CachePolicy::Revalidate,
            _ => CachePolicy::NoStore,
        }
    }
}

impl From<QueryError> for Error {
    fn from(error: QueryError) -> Self {
        let (status, code) = error_details(&error);
        let message = match &error {
            QueryError::SeriesNotFound(not_found) => series_not_found(not_found),
            error => error.to_string(),
        };
        Self {
            status,
            code,
            message,
        }
    }
}

/// A missing series' message and where its change is said: for a name written as an id, its own answer (one written
/// otherwise, or cut short for display, was never an id).
fn series_not_found(not_found: &SeriesNotFound) -> String {
    let series = not_found.series();
    let id = (!series.is_empty()
        && series
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'))
    .then_some(series);
    format!("{not_found} {}", changes_hint(id))
}

/// Errors from below the query (node calls, IO) take the query's translation, except a
/// transaction the node refused, which is the client's input.
impl From<BrkError> for Error {
    fn from(error: BrkError) -> Self {
        match error {
            BrkError::TxRejected(reason) => Self::bad_request(reason),
            error => QueryError::from(error).into(),
        }
    }
}

impl OperationOutput for Error {
    type Inner = ();
}

/// Internal failures are logged, and their details (paths, node advice, panic payloads) stay out
/// of the body.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let policy = self.cache_policy();
        let message = if self.status == StatusCode::INTERNAL_SERVER_ERROR {
            error!("{}", self.message);
            "Internal server error".to_owned()
        } else {
            self.message
        };
        let body = to_vec(&ErrorBody::new(
            error_type(self.status),
            self.code,
            message,
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
        policy.apply(response.headers_mut());
        response
    }
}
