use schemars::JsonSchema;
use serde::Deserialize;

/// Where a reader of the log left off.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LogsQuery {
    /// The `started_unix_ms` of the last answer: `after` counts in that process's log.
    pub started_unix_ms: Option<i64>,
    /// Only the lines after this `seq`, when `started_unix_ms` is this process's; otherwise every kept line.
    pub after: Option<u64>,
}
