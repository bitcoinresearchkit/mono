use brk_types::Height;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Blocks' payments read as round-dollar amounts: the prices they imply, counted in fine buckets.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OraclePayments {
    /// A bucket's width in oracle bins (a bin is 1/200 of a decade of price).
    pub step: f64,
    pub blocks: Vec<BlockPayments>,
}

/// One block's payments, each read as each of the oracle's 19 round-dollar amounts ($1 to $10,000)
/// at its exact amount in sats, counted by the price each reading implies.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BlockPayments {
    pub height: Height,
    /// The first bucket: bucket `i` holds the readings whose $100 bin is in `[i, i + 1) * step`,
    /// the price `10^(10 - i * step / 200)` dollars at its edge.
    pub first: u32,
    /// The readings in each bucket from `first` on, over the oracle's search window for the block.
    pub counts: Vec<u32>,
}
