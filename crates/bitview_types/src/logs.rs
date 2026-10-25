use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The server's last log lines, oldest first.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Logs {
    /// When this server process began its log, in Unix milliseconds: another than the last answer's shows the
    /// server restarted.
    pub started_unix_ms: i64,
    pub lines: Vec<LogLine>,
    /// Where this answer ends: sent back as `after`, the lines since. `{started_unix_ms}-{seq}`, the process and the
    /// `seq` of the next line it will log; to pass back as it is, not to read.
    pub next: String,
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
