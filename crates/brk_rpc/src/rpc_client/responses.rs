//! The fields Bitview reads of Bitcoin Core's RPC answers, under Core's names. serde skips
//! every other field, so Core adding or reshaping those changes nothing here.

use bitcoin::BlockHash;
use serde::Deserialize;

/// `getblockcount`: the best chain's height.
#[derive(Deserialize)]
pub struct GetBlockCount(pub u64);

/// `getblockheader`, verbose.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GetBlockHeaderVerbose {
    /// Zero or less when the block is off the best chain.
    pub confirmations: i64,
    pub height: i64,
    /// `None` at genesis.
    #[serde(rename = "previousblockhash")]
    pub previous_block_hash: Option<BlockHash>,
}

/// The shared `getchaintips` record, preserving the public RPC name and field types.
pub use brk_types::ChainTip as GetChainTip;

/// `getblocktemplate`.
#[derive(Deserialize)]
pub struct GetBlockTemplate {
    #[serde(rename = "previousblockhash")]
    pub previous_block_hash: String,
    pub transactions: Vec<BlockTemplateTransaction>,
}

/// One of `getblocktemplate`'s transactions.
#[derive(Deserialize)]
pub struct BlockTemplateTransaction {
    /// The serialized transaction, in hex.
    pub data: String,
    pub txid: String,
    /// The wtxid.
    pub hash: String,
    /// The 1-based positions in the template of the transactions it spends.
    pub depends: Vec<i64>,
    pub fee: i64,
    pub sigops: i64,
    pub weight: u64,
}

/// `getmempoolinfo`.
#[derive(Deserialize)]
pub struct GetMempoolInfo {
    /// In BTC per kvB.
    #[serde(rename = "mempoolminfee")]
    pub mempool_min_fee: f64,
    /// Whether the node finished loading its saved mempool (false while it reloads after a
    /// restart: the listing then lacks what it hasn't read back yet).
    pub loaded: bool,
}

/// `getblockchaininfo`.
#[derive(Deserialize)]
pub struct GetBlockchainInfo {
    pub chain: String,
    pub blocks: i64,
    pub headers: i64,
}
