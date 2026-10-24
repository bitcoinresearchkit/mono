//! The fields Bitview reads of Bitcoin Core's RPC answers, under Core's names. serde skips
//! every other field, so Core adding or reshaping those changes nothing here.

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
    /// None for the genesis block.
    #[serde(rename = "previousblockhash")]
    pub(crate) previous_block_hash: Option<String>,
}

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
}

/// `getblockchaininfo`.
#[derive(Deserialize)]
pub struct GetBlockchainInfo {
    pub chain: String,
    pub blocks: i64,
    pub headers: i64,
}
