use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Timestamp;

/// UNIX timestamp path parameter
#[derive(Deserialize, JsonSchema)]
pub struct TimestampParam {
    pub timestamp: Timestamp,
}
