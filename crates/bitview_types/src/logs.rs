use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The server's last log lines, oldest first.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Logs {
    /// When this server process began its log, in Unix milliseconds: a `seq` counts within it.
    pub started_unix_ms: i64,
    pub lines: Vec<LogLine>,
}

/// One line of the server's log.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LogLine {
    /// Its place in this process's log, from 0 with no gaps.
    pub seq: u64,
    /// When it was logged, in Unix milliseconds.
    pub unix_ms: i64,
    pub level: LogLevel,
    /// What the log files show after the level: a request's status, path and duration, or a message and its fields.
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
