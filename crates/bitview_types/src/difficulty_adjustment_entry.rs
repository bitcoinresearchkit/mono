use std::borrow::Cow;

use brk_types::{Height, Timestamp};
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A difficulty adjustment, written as an array (as mempool.space does): `[timestamp, height,
/// difficulty, adjustment]`.
#[derive(Debug)]
pub struct DifficultyAdjustmentEntry {
    /// Unix timestamp of the adjustment
    pub timestamp: Timestamp,
    pub height: Height,
    pub difficulty: f64,
    /// Adjustment ratio (new/previous, e.g. 1.068 = +6.8%)
    pub adjustment: f64,
}

impl Serialize for DifficultyAdjustmentEntry {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (
            self.timestamp,
            self.height,
            self.difficulty,
            self.adjustment,
        )
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DifficultyAdjustmentEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (timestamp, height, difficulty, adjustment) = Deserialize::deserialize(deserializer)?;
        Ok(Self {
            timestamp,
            height,
            difficulty,
            adjustment,
        })
    }
}

impl JsonSchema for DifficultyAdjustmentEntry {
    fn schema_name() -> Cow<'static, str> {
        "DifficultyAdjustmentEntry".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "A difficulty adjustment, written as an array (as mempool.space does): \
                `[timestamp, height, difficulty, adjustment]`.",
            "type": "array",
            "prefixItems": [
                generator.subschema_for::<Timestamp>(),
                generator.subschema_for::<Height>(),
                {
                    "description": "Difficulty after the adjustment",
                    "type": "number",
                    "format": "double",
                    "examples": [110_451_832_649_830.94]
                },
                {
                    "description": "Adjustment ratio (new/previous, e.g. 1.068 = +6.8%)",
                    "type": "number",
                    "format": "double",
                    "examples": [1.068]
                }
            ],
            "items": false,
            "minItems": 4,
        })
    }
}
