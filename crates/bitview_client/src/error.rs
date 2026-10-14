use std::fmt;

use bitview_types::ErrorCode;

/// Error type for Bitview client operations.
#[derive(Debug)]
pub struct BitviewError {
    /// HTTP status of the server's error answer.
    pub status: Option<u16>,
    /// The server's machine-readable code, from its error body.
    pub code: Option<ErrorCode>,
    pub message: String,
}

impl BitviewError {
    /// A client-side error, not an answer from the server.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            status: None,
            code: None,
            message: message.into(),
        }
    }
}

impl fmt::Display for BitviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for BitviewError {}

/// Result using the Bitview client's error type.
pub type Result<T> = std::result::Result<T, BitviewError>;
