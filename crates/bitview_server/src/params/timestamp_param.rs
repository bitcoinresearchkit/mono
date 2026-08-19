use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Timestamp;

/// UNIX timestamp path parameter
#[derive(Deserialize, JsonSchema)]
pub struct TimestampParam {
    pub timestamp: Timestamp,
}

/// Optional UNIX timestamp query parameter
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OptionalTimestampParam {
    pub timestamp: Option<Timestamp>,
}
