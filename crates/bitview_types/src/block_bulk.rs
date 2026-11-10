use bitview_primitives::PoolSlug;
use brk_types::{Address, BlockHash, FeeRate, Height, Hex, Sats, Timestamp, Weight};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{BlockOrphan, MerkleNode};

/// Named bulk percentile fields in upstream's min, p10, p25, p50, p75, p90, max order.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlockPercentiles<T> {
    pub min: T,
    pub perc_10: T,
    pub perc_25: T,
    pub perc_50: T,
    pub perc_75: T,
    pub perc_90: T,
    pub max: T,
}

impl<T> From<[T; 7]> for BlockPercentiles<T> {
    fn from([min, perc_10, perc_25, perc_50, perc_75, perc_90, max]: [T; 7]) -> Self {
        Self { min, perc_10, perc_25, perc_50, perc_75, perc_90, max }
    }
}

impl<T> BlockPercentiles<T> {
    pub fn into_array(self) -> [T; 7] {
        let Self { min, perc_10, perc_25, perc_50, perc_75, perc_90, max } = self;
        [min, perc_10, perc_25, perc_50, perc_75, perc_90, max]
    }
}

/// Flat bulk block wire shape, using the same local facts and semantics as ordinary V1 rows.
/// Effective rate distributions are vsize-weighted locally; upstream ranks by tx count.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlockBulk {
    pub height: Height,
    pub hash: BlockHash,
    pub timestamp: Timestamp,
    pub median_timestamp: Timestamp,
    pub previous_block_hash: Option<BlockHash>,
    pub difficulty: f64,
    pub header: Hex,
    pub version: u32,
    pub bits: u32,
    pub nonce: u32,
    pub size: u64,
    pub weight: Weight,
    /// All transactions, including the coinbase.
    pub tx_count: u32,
    pub merkle_root: MerkleNode,
    /// Actual coinbase output value, including any claimed fees.
    pub reward: Sats,
    pub total_fee_amt: Sats,
    pub avg_fee_amt: Sats,
    pub median_fee_amt: Sats,
    pub fee_amt_percentiles: BlockPercentiles<Sats>,
    pub avg_fee_rate: FeeRate,
    /// Local effective rate median by vsize, not upstream's 4M WU middle band.
    pub median_fee_rate: FeeRate,
    /// Local effective rate percentiles by vsize, not upstream's count-ranked percentiles.
    pub fee_rate_percentiles: BlockPercentiles<FeeRate>,
    /// Inputs excluding the coinbase input.
    pub total_inputs: u64,
    pub total_input_amt: Sats,
    /// All outputs, including coinbase and OP_RETURN outputs.
    pub total_outputs: u64,
    /// Output value excluding the coinbase outputs.
    pub total_output_amt: Sats,
    /// Witness transactions excluding the coinbase.
    pub segwit_total_txs: u32,
    pub segwit_total_size: u64,
    pub segwit_total_weight: Weight,
    /// Non-coinbase bytes / all tx count, rounded to two decimals.
    pub avg_tx_size: f64,
    pub utxoset_change: i64,
    /// Local UTXO count excludes OP_RETURN, but includes scripts over 10,000 bytes.
    pub utxoset_size: u64,
    /// Coinbase scriptsig hex, not the whole serialized transaction.
    pub coinbase_raw: Hex,
    pub coinbase_address: Option<Address>,
    pub coinbase_addresses: Vec<Address>,
    pub coinbase_signature: String,
    pub coinbase_signature_ascii: String,
    pub pool_slug: PoolSlug,
    /// Upstream pools.unique_id, zero-based; not pools.id or the one-based mining API ID.
    pub pool_id: u8,
    /// This node's published stale-block observations at this height.
    pub orphans: Vec<BlockOrphan>,
}
