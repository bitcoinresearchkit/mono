use bitview_query::{Error, Result};
use bitview_types::TimestampRange;
use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::Deserialize;

/// Two required inclusive UNIX header timestamps, in canonical unsigned decimal
/// seconds (0..=4294967295); unknown or repeated fields are rejected.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimestampRangeQuery {
    #[serde(deserialize_with = "bitview_types::de_plain")]
    pub from: Timestamp,
    #[serde(deserialize_with = "bitview_types::de_plain")]
    pub to: Timestamp,
}

impl TimestampRangeQuery {
    pub fn range(self) -> Result<TimestampRange> {
        TimestampRange::new(self.from, self.to)
            .ok_or_else(|| Error::InvalidParam("from must be <= to".into()))
    }
}
