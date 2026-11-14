use std::collections::BTreeMap;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{BlockHash, Height, Sats, SigOps, Timestamp, Txid, Weight};

/// The next block's template in BIP 22/23 form, as Bitcoin Core's `getblocktemplate` answers
/// (`rules: ["segwit"]`): what pools and mining software read. Complete once a miner adds a
/// coinbase paying `coinbasevalue` and carrying `default_witness_commitment`. Returned by
/// `GET /api/block-template`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct BlockTemplate {
    /// The block version: BIP 9 version bits, no deployment signaled.
    pub version: i32,
    /// The rules the template follows: `!` marks one a client must understand.
    pub rules: Vec<String>,
    /// BIP 9 deployments the client may signal: none.
    pub vbavailable: BTreeMap<String, u32>,
    /// Version bits the client must set: none.
    pub vbrequired: u32,
    pub previousblockhash: BlockHash,
    /// The transactions, every parent before its children.
    pub transactions: Vec<BlockTemplateTransaction>,
    /// Data to put in the coinbase's script: none.
    pub coinbaseaux: BTreeMap<String, String>,
    /// What the coinbase may pay: the subsidy at `height` plus every transaction's fee (sats).
    pub coinbasevalue: Sats,
    /// The proof-of-work target, as 64 hex digits.
    pub target: String,
    /// The earliest valid `curtime`: the median time past plus one second.
    pub mintime: Timestamp,
    /// What the client may change: the time, the transactions, the previous block.
    pub mutable: Vec<String>,
    pub noncerange: String,
    /// Consensus limits for the whole block, coinbase included.
    pub sigoplimit: u32,
    pub sizelimit: u32,
    pub weightlimit: u32,
    /// The block time to use: now, or `mintime` if later.
    pub curtime: Timestamp,
    /// The compact target, as 8 hex digits.
    pub bits: String,
    pub height: Height,
    /// The coinbase output committing to the transactions' witnesses (BIP 141), as script hex.
    pub default_witness_commitment: String,
}

/// One transaction of a [`BlockTemplate`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct BlockTemplateTransaction {
    /// The transaction serialized with its witness, as hex.
    pub data: String,
    pub txid: Txid,
    /// The witness transaction id.
    pub hash: String,
    /// The 1-based positions in `transactions` of the ones it spends.
    pub depends: Vec<u32>,
    /// Its fee (sats).
    pub fee: Sats,
    /// Its signature-operation cost.
    pub sigops: SigOps,
    pub weight: Weight,
}
