use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Timestamp;

#[derive(Deserialize, JsonSchema)]
pub struct TimestampParam {
    pub timestamp: Timestamp,
}
