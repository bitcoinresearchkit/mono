//! BlindBit oracle v1's HTTP responses: the API silent payments light wallets speak (spdk and Dana,
//! Bull Bitcoin, blindbitd).

use std::fmt;

use brk_types::{BlockHash, Height, Hex, Sats, SpentOutputHash, Timestamp, Txid, Vout};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Which of a block's two filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum BlindbitFilterKind {
    /// Its new Taproot output keys.
    NewUtxos,
    /// Its spends of Taproot outputs.
    Spent,
}

impl fmt::Display for BlindbitFilterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NewUtxos => "new-utxos",
            Self::Spent => "spent",
        })
    }
}

/// What the server serves.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlindbitInfo {
    /// Bitcoin Core's chain name: `main`.
    pub network: String,
    /// The last indexed block.
    pub height: Height,
    /// Whether only tweaks are served: `false`, filters and UTXOs are too.
    pub tweaks_only: bool,
    /// Whether the full tweak index is served without dust limits: `false`, it takes them.
    pub tweaks_full_basic: bool,
    /// Whether the full tweak index takes a dust limit: `true`.
    pub tweaks_full_with_dust_filter: bool,
    /// Whether cut-through tweaks (of transactions with an unspent Taproot output) take a dust
    /// limit: `true`.
    pub tweaks_cut_through_with_dust_filter: bool,
}

/// The last indexed block's height.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlindbitBlockHeight {
    pub block_height: Height,
}

/// A block's hash.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlindbitBlockHash {
    pub block_hash: BlockHash,
}

/// A BIP158 Golomb-coded set (the basic filters' parameters, keyed by the block hash).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlindbitFilter {
    pub block_hash: BlockHash,
    pub block_height: Height,
    /// The filter: the element count, then the coded deltas.
    pub data: Hex,
    /// Always 4, BlindBit's Taproot-only filters.
    pub filter_type: u8,
}

/// A block's spends of Taproot outputs.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlindbitSpentIndex {
    pub block_hash: BlockHash,
    pub data: Vec<SpentOutputHash>,
}

/// A Taproot output of an eligible transaction.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlindbitUtxo {
    pub txid: Txid,
    pub vout: Vout,
    pub value: Sats,
    /// Its script, `OP_1` and a 32-byte push of the output key.
    pub scriptpubkey: Hex,
    pub block_height: Height,
    pub block_hash: BlockHash,
    /// The block's header time.
    pub timestamp: Timestamp,
    /// Whether a published block spends it.
    pub spent: bool,
}
