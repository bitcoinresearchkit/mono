use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Timestamp;

#[derive(Deserialize, JsonSchema)]
pub struct TimestampParam {
    #[serde(deserialize_with = "bitview_types::de_plain")]
    pub timestamp: Timestamp,
}
