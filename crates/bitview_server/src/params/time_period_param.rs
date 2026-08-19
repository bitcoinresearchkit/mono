use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::TimePeriod;

/// Time period path parameter (24h, 3d, 1w, 1m, 3m, 6m, 1y, 2y, 3y)
#[derive(Deserialize, JsonSchema)]
pub struct TimePeriodParam {
    #[schemars(example = &"24h")]
    pub time_period: TimePeriod,
}
