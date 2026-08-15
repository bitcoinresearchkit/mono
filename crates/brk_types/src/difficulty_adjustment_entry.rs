use schemars::JsonSchema;
use serde::ser::SerializeTuple;
use serde::{Deserialize, Serialize, Serializer};

use crate::{Height, Timestamp};

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
        let mut tup = serializer.serialize_tuple(4)?;
        tup.serialize_element(&self.timestamp)?;
        tup.serialize_element(&self.height)?;
        tup.serialize_element(&self.difficulty)?;
        tup.serialize_element(&self.change_percent)?;
        tup.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_schema_is_a_fixed_four_item_array() {
        let schema =
            serde_json::to_value(schemars::schema_for!(DifficultyAdjustmentEntry)).unwrap();
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["minItems"], 4);
        assert_eq!(schema["maxItems"], 4);
        assert_eq!(schema["items"]["type"], "number");
    }
}
