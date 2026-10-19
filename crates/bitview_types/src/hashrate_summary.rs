use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{DifficultyEntry, HashrateEntry};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HashrateSummary {
    pub hashrates: Vec<HashrateEntry>,
    pub difficulty: Vec<DifficultyEntry>,
    /// Current network hashrate over the trailing 7 days (H/s)
    #[serde(rename = "currentHashrate")]
    #[schemars(example = 700_000_000_000_000_000_000_u128)]
    pub current_hashrate: u128,
    #[serde(rename = "currentDifficulty")]
    #[schemars(example = 110_451_832_649_830.94)]
    pub current_difficulty: f64,
}
