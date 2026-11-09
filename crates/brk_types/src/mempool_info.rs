#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{FeeRate, Sats, VSize};

/// Mempool statistics with incrementally maintained fee histogram.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct MempoolInfo {
    /// Number of transactions in the mempool
    #[cfg_attr(feature = "schemars", schemars(example = 5000))]
    pub count: usize,
    /// Total virtual size of all transactions in the mempool (vbytes), sigop-adjusted as Bitcoin
    /// Core's `getmempoolinfo` `bytes` and every fee rate measure it
    pub vsize: VSize,
    /// Total fees of all transactions in the mempool (satoshis)
    pub total_fee: Sats,
    /// Fee histogram as Esplora bins it: `[[fee_rate, vsize], ...]` by descending fee rate, each
    /// entry the vsize of the transactions paying from its fee rate up to the previous entry's,
    /// a bin closing at the first fee rate change past 50,000 vB
    #[cfg_attr(feature = "schemars", schemars(with = "Vec<[f64; 2]>"))]
    pub fee_histogram: Vec<(FeeRate, VSize)>,
}
