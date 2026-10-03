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

    /// Estimated time until retarget (milliseconds)
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

    /// Time-adjusted average (milliseconds)
    #[schemars(example = 580000_u64)]
    pub adjusted_time_avg: u64,

    /// Time offset from expected schedule (seconds)
    #[schemars(example = 0)]
    pub time_offset: i64,

    /// Expected blocks based on wall clock time since epoch start
    #[schemars(example = 1827.21)]
    pub expected_blocks: f64,
}
