use schemars::JsonSchema;
use serde::Deserialize;

/// Where a reader of the log left off.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LogsQuery {
    /// The last answer's `next`: only the lines since. From a process that's since restarted, or none, every kept line.
    pub after: Option<String>,
}
