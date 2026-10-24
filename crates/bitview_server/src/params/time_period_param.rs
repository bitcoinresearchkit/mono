use bitview_types::TimePeriod;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct TimePeriodParam {
    /// Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.
    #[schemars(example = &"24h")]
    pub time_period: TimePeriod,
}
