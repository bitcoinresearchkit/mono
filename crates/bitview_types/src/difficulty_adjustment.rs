use brk_types::{Height, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Difficulty adjustment information.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DifficultyAdjustment {
    /// Progress through current difficulty epoch (0-100%)
    #[schemars(example = 44.4)]
    pub progress_percent: f64,

    /// Estimated difficulty change at next retarget (%)
    #[schemars(example = 2.5)]
    pub difficulty_change: f64,

    /// Estimated timestamp of next retarget (milliseconds)
    #[schemars(example = 1627762478000_u64)]
    pub estimated_retarget_date: u64,

    /// Blocks remaining until retarget
    #[schemars(example = 1121)]
    pub remaining_blocks: u32,

    /// Estimated time until retarget at `adjustedTimeAvg` (milliseconds)
    #[schemars(example = 665977000_u64)]
    pub remaining_time: u64,

    /// Previous difficulty adjustment (%)
    #[schemars(example = -4.8)]
    pub previous_retarget: f64,

    /// Timestamp of most recent retarget (seconds)
    #[schemars(example = 1627000000_u64)]
    pub previous_time: Timestamp,

    /// Height of next retarget
    #[schemars(example = 741888)]
    pub next_retarget_height: Height,

    /// Average block time in current epoch (milliseconds)
    #[schemars(example = 580000_u64)]
    pub time_avg: u64,

    /// Block time the projection uses (milliseconds): `timeAvg`, except in an epoch's first 503
    /// blocks, where it spans the last 503 blocks, those before the retarget scaled by it
    #[schemars(example = 580000_u64)]
    pub adjusted_time_avg: u64,

    /// Testnet's offset for its 20-minute minimum-difficulty rule (milliseconds): always 0 on
    /// mainnet, as mempool.space
    #[schemars(example = 0)]
    pub time_offset: i64,

    /// Blocks expected at the 10-minute target between the epoch's first block and the tip block
    #[schemars(example = 1827.21)]
    pub expected_blocks: f64,
}
