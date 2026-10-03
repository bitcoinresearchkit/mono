use brk_types::{Height, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A single difficulty data point in the hashrate summary.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DifficultyEntry {
    /// Unix timestamp of the difficulty adjustment
    pub time: Timestamp,
    /// Block height of the adjustment
    pub height: Height,
    /// Difficulty value
    #[schemars(example = 110_451_832_649_830.94)]
    pub difficulty: f64,
    /// Adjustment ratio (new/previous, e.g. 1.068 = +6.8%)
    #[schemars(example = 1.068)]
    pub adjustment: f64,
}
