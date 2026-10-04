use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The JSON body of every API error (`application/problem+json`).
#[derive(Debug, Serialize, JsonSchema)]
pub struct ErrorBody {
    error: ErrorDetail,
}

impl ErrorBody {
    pub fn new(r#type: ErrorType, code: ErrorCode, message: String, doc_url: &'static str) -> Self {
        Self {
            error: ErrorDetail {
                r#type,
                code,
                message,
                doc_url,
            },
        }
    }
}

#[derive(Debug, Serialize, JsonSchema)]
struct ErrorDetail {
    /// Error category, following the HTTP status
    r#type: ErrorType,
    /// Machine-readable error code
    code: ErrorCode,
    /// Human-readable description
    message: String,
    /// Link to API documentation
    #[schemars(with = "String")]
    doc_url: &'static str,
}

/// Error category, following the HTTP status: `invalid_request` (4xx other than 404),
/// `not_found` (404), `unavailable` (503; `Retry-After` when transient), `timeout` (504), `internal`
/// (other 5xx).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    InvalidRequest,
    NotFound,
    Unavailable,
    Timeout,
    Internal,
}

/// Machine-readable error code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    NotFound,
    InvalidAddr,
    InvalidNetwork,
    UnsupportedType,
    ParseError,
    NoSeries,
    SeriesUnsupportedIndex,
    WeightExceeded,
    TooManyUtxos,
    UnknownAddr,
    UnknownTxid,
    OutOfRange,
    UnindexableDate,
    NoData,
    SeriesNotFound,
    MempoolNotAvailable,
    StateUpdating,
    InternalError,
    BadRequest,
    Overloaded,
    Timeout,
    MethodNotAllowed,
}

impl ErrorCode {
    /// Conditions that clear on their own; responses carry `Retry-After`.
    pub const fn is_transient(self) -> bool {
        matches!(self, Self::StateUpdating | Self::Overloaded)
    }
}
