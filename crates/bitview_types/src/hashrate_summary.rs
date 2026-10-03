use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{DifficultyEntry, HashrateEntry};

/// Summary of network hashrate and difficulty data.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HashrateSummary {
    /// Historical hashrate data points
    pub hashrates: Vec<HashrateEntry>,
    /// Historical difficulty adjustments
    pub difficulty: Vec<DifficultyEntry>,
    /// Current network hashrate (H/s)
    #[serde(rename = "currentHashrate")]
    #[schemars(example = 700_000_000_000_000_000_000_u128)]
    pub current_hashrate: u128,
    /// Current network difficulty
    #[serde(rename = "currentDifficulty")]
    #[schemars(example = 110_451_832_649_830.94)]
    pub current_difficulty: f64,
}
