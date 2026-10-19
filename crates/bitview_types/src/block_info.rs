use brk_types::{BlockHash, Height, Timestamp, Weight};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Block information matching mempool.space /api/block/{hash}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlockInfo {
    /// Block hash
    pub id: BlockHash,
    pub height: Height,
    #[schemars(example = 536870912)]
    pub version: u32,
    /// Block timestamp (Unix time)
    pub timestamp: Timestamp,
    /// Compact target (bits)
    #[schemars(example = 386089497)]
    pub bits: u32,
    #[schemars(example = 2083236893)]
    pub nonce: u32,
    #[schemars(example = 110_451_832_649_830.94)]
    pub difficulty: f64,
    #[schemars(
        example = &"4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"
    )]
    pub merkle_root: String,
    #[schemars(example = 2500)]
    pub tx_count: u32,
    /// Block size in bytes
    #[schemars(example = 1580000)]
    pub size: u64,
    /// Block weight in weight units
    pub weight: Weight,
    #[serde(rename = "previousblockhash")]
    pub previous_block_hash: BlockHash,
    /// Median time of the last 11 blocks
    #[serde(rename = "mediantime")]
    pub median_time: Timestamp,
}
