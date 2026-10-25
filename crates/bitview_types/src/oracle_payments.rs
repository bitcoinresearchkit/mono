use brk_types::{BlockHash, Height};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 144 blocks' payments, each read as each of the oracle's 19 round-dollar amounts ($1 to
/// $10,000) at its exact amount in sats, counted by the price each reading implies in buckets a
/// tenth of the oracle's bins.
///
/// A reading of `sats` as `amount` dollars implies $100 lands at bin
/// `(log10(sats) + 2 - log10(amount)) * 200`; bucket `b` holds the readings in bins
/// `[b * step, (b + 1) * step)`, prices from `10^(10 - (b + 1) * step / 200)` up to
/// `10^(10 - b * step / 200)` dollars.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OraclePayments {
    /// The last block's hash: it commits to every block before it.
    pub hash: BlockHash,
    /// A bucket's width in oracle bins: 0.1, a price range of 0.115%.
    pub step: f64,
    /// Each block's, in order.
    pub blocks: Vec<OracleBlockPayments>,
}

/// One block's payments read as round dollars, over the oracle's search window for it.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OracleBlockPayments {
    pub height: Height,
    /// The first bucket's index: the window's first bin's lower edge (half a bin under it) over
    /// `step`.
    pub first_bucket: u32,
    /// The readings in each bucket from `first_bucket` on: bucket `first_bucket + k` at index `k`.
    pub counts: Vec<u32>,
}
