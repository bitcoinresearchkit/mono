use brk_types::{Dollars, FeeRate, Sats, Weight};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::BlockPool;

/// Extended block data matching mempool.space /api/v1/blocks extras
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlockExtras {
    /// Total fees in satoshis
    #[serde(rename = "totalFees")]
    pub total_fees: Sats,

    /// Median fee rate in sat/vB
    #[serde(rename = "medianFee")]
    pub median_fee: FeeRate,

    /// Fee rate range: [min, 10%, 25%, 50%, 75%, 90%, max]
    #[serde(rename = "feeRange")]
    pub fee_range: [FeeRate; 7],

    /// Total block reward (subsidy + fees) in satoshis
    pub reward: Sats,

    pub pool: BlockPool,

    /// Mean fee per non-coinbase transaction in satoshis, rounded down (0 without any)
    #[serde(rename = "avgFee")]
    pub avg_fee: Sats,

    /// Average fee rate in sat/vB
    #[serde(rename = "avgFeeRate")]
    pub avg_fee_rate: FeeRate,

    /// Raw coinbase transaction scriptsig as hex
    #[serde(rename = "coinbaseRaw")]
    pub coinbase_raw: String,

    /// Primary coinbase output address
    #[serde(rename = "coinbaseAddress")]
    pub coinbase_address: Option<String>,

    /// All coinbase output addresses
    #[serde(rename = "coinbaseAddresses")]
    pub coinbase_addresses: Vec<String>,

    /// Coinbase output script in ASM format
    #[serde(rename = "coinbaseSignature")]
    pub coinbase_signature: String,

    /// Coinbase scriptsig with each byte read as the Unicode code point of the same value
    #[serde(rename = "coinbaseSignatureAscii")]
    pub coinbase_signature_ascii: String,

    /// Non-coinbase transaction bytes over the transaction count (coinbase included), rounded to
    /// 2 decimals
    #[serde(rename = "avgTxSize")]
    #[schemars(example = 534.5)]
    pub avg_tx_size: f64,

    /// Total number of inputs (excluding coinbase)
    #[serde(rename = "totalInputs")]
    #[schemars(example = 5000)]
    pub total_inputs: u64,

    #[serde(rename = "totalOutputs")]
    #[schemars(example = 7500)]
    pub total_outputs: u64,

    /// Total output amount in satoshis, coinbase excluded
    #[serde(rename = "totalOutputAmt")]
    pub total_output_amt: Sats,

    /// Median fee amount in satoshis
    #[serde(rename = "medianFeeAmt")]
    pub median_fee_amt: Sats,

    /// Fee amount percentiles in satoshis: [min, 10%, 25%, 50%, 75%, 90%, max]
    #[serde(rename = "feePercentiles")]
    pub fee_percentiles: [Sats; 7],

    #[serde(rename = "segwitTotalTxs")]
    #[schemars(example = 2000)]
    pub segwit_total_txs: u32,

    /// Total size of segwit transactions in bytes
    #[serde(rename = "segwitTotalSize")]
    #[schemars(example = 1200000)]
    pub segwit_total_size: u64,

    #[serde(rename = "segwitTotalWeight")]
    pub segwit_total_weight: Weight,

    /// Raw 80-byte block header as hex
    pub header: String,

    /// UTXO set change as mempool.space and bitcoin-cli report it: total outputs minus total
    /// inputs, unspendable outputs (OP_RETURN) included, unlike `utxoSetSize`
    #[serde(rename = "utxoSetChange")]
    #[schemars(example = 2500)]
    pub utxo_set_change: i64,

    /// Total spendable UTXO set size at this height (excludes OP_RETURN and other unspendable outputs)
    #[serde(rename = "utxoSetSize")]
    #[schemars(example = 170_000_000)]
    pub utxo_set_size: u64,

    /// Total input amount in satoshis
    #[serde(rename = "totalInputAmt")]
    pub total_input_amt: Sats,

    /// Virtual size in vbytes: weight / 4, unrounded
    #[serde(rename = "virtualSize")]
    #[schemars(example = 998000.25)]
    pub virtual_size: f64,

    /// Timestamp when the block was first seen (always null, not yet supported)
    #[serde(rename = "firstSeen")]
    pub first_seen: Option<u64>,

    /// Orphaned blocks (always empty)
    pub orphans: Vec<String>,

    /// USD price at block height
    pub price: Dollars,
}
