use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Time period for mining statistics.
///
/// Used to specify the lookback window for pool statistics, hashrate calculations,
/// and other time-based mining series.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum TimePeriod {
    #[default]
    #[serde(rename = "24h")]
    Day,
    #[serde(rename = "3d")]
    ThreeDays,
    #[serde(rename = "1w")]
    Week,
    #[serde(rename = "1m")]
    Month,
    #[serde(rename = "3m")]
    ThreeMonths,
    #[serde(rename = "6m")]
    SixMonths,
    #[serde(rename = "1y")]
    Year,
    #[serde(rename = "2y")]
    TwoYears,
    #[serde(rename = "3y")]
    ThreeYears,
    #[serde(rename = "all")]
    All,
}

impl fmt::Display for TimePeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimePeriod::Day => write!(f, "24h"),
            TimePeriod::ThreeDays => write!(f, "3d"),
            TimePeriod::Week => write!(f, "1w"),
            TimePeriod::Month => write!(f, "1m"),
            TimePeriod::ThreeMonths => write!(f, "3m"),
            TimePeriod::SixMonths => write!(f, "6m"),
            TimePeriod::Year => write!(f, "1y"),
            TimePeriod::TwoYears => write!(f, "2y"),
            TimePeriod::ThreeYears => write!(f, "3y"),
            TimePeriod::All => write!(f, "all"),
        }
    }
}
