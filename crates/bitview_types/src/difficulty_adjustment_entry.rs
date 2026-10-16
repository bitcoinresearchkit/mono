use brk_types::{Height, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, Serializer};

/// A single difficulty adjustment entry.
/// Serializes as array: [timestamp, height, difficulty, change_percent]
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(with = "[f64; 4]")]
pub struct DifficultyAdjustmentEntry {
    /// Unix timestamp of the adjustment
    pub timestamp: Timestamp,
    /// Block height of the adjustment
    pub height: Height,
    /// Difficulty value
    #[schemars(example = 110_451_832_649_830.94)]
    pub difficulty: f64,
    /// Adjustment ratio (new/previous, e.g. 1.068 = +6.8%)
    #[schemars(example = 1.068)]
    pub change_percent: f64,
}

impl Serialize for DifficultyAdjustmentEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (
            self.timestamp,
            self.height,
            self.difficulty,
            self.change_percent,
        )
            .serialize(serializer)
    }
}
