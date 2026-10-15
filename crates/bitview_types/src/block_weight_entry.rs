use brk_types::{Height, Timestamp, Weight};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A single block weight data point.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlockWeightEntry {
    /// Average block height in this bucket
    #[schemars(example = 890621)]
    pub avg_height: Height,
    /// Average block timestamp in this bucket
    #[schemars(example = 1743631892)]
    pub timestamp: Timestamp,
    /// Mean block weight in this bucket, rounded half up (weight units)
    #[schemars(example = 3990000)]
    pub avg_weight: Weight,
}
