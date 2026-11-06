#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{FeeRate, Sats, Weight};

/// Block info in a mempool.space like format for fee estimation.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MempoolBlock {
    /// Sum of the transactions' serialized sizes in bytes (witness + non-witness), without header
    /// or coinbase.
    #[cfg_attr(feature = "schemars", schemars(example = 1604417))]
    pub block_size: u64,

    /// The transactions' weight over 4, unrounded (mempool.space's): the block's virtual size
    #[cfg_attr(feature = "schemars", schemars(example = 998368.25))]
    pub block_v_size: f64,

    /// Number of transactions in the projected block
    #[cfg_attr(feature = "schemars", schemars(example = 863))]
    pub n_tx: u32,

    /// Total fees in satoshis
    #[cfg_attr(feature = "schemars", schemars(example = 8875608))]
    pub total_fees: Sats,

    /// Median fee rate in sat/vB
    #[cfg_attr(feature = "schemars", schemars(example = 10.5))]
    pub median_fee: FeeRate,

    /// Vsize-weighted percentiles of the transactions' CPFP chunk fee rates, in sat/vB:
    /// [0, 10, 25, 50, 75, 90, 100] for the first block, [5, 10, 25, 50, 75, 90, 95] for the
    /// later projected ones
    #[cfg_attr(feature = "schemars", schemars(example = example_fee_range()))]
    pub fee_range: [FeeRate; 7],
}

#[cfg(feature = "schemars")]
fn example_fee_range() -> [FeeRate; 7] {
    [
        FeeRate::new(1.0),
        FeeRate::new(2.42),
        FeeRate::new(8.1),
        FeeRate::new(10.14),
        FeeRate::new(11.05),
        FeeRate::new(12.04),
        FeeRate::new(302.11),
    ]
}

impl MempoolBlock {
    pub fn new(
        tx_count: u32,
        total_size: u64,
        total_weight: Weight,
        total_fee: Sats,
        fee_range: [FeeRate; 7],
    ) -> Self {
        Self {
            block_size: total_size,
            block_v_size: u64::from(total_weight) as f64 / 4.0,
            n_tx: tx_count,
            total_fees: total_fee,
            median_fee: fee_range[3],
            fee_range,
        }
    }
}
