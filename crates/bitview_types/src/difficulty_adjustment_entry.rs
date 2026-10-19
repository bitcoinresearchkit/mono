use brk_types::{Height, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, Serializer};

/// Serializes as an array: [timestamp, height, difficulty, adjustment ratio].
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(with = "[f64; 4]")]
pub struct DifficultyAdjustmentEntry {
    /// Unix timestamp of the adjustment
    pub timestamp: Timestamp,
    pub height: Height,
    #[schemars(example = 110_451_832_649_830.94)]
    pub difficulty: f64,
    /// Adjustment ratio (new/previous, e.g. 1.068 = +6.8%)
    #[serde(rename = "change_percent")]
    #[schemars(example = 1.068)]
    pub adjustment: f64,
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
            self.adjustment,
        )
            .serialize(serializer)
    }
}
