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
    /// The header's time field.
    pub time: u32,
    /// `None` at genesis.
    #[serde(rename = "previousblockhash")]
    pub previous_block_hash: Option<BlockHash>,
}

/// One of `getchaintips`' tips.
#[derive(Debug, Clone, Deserialize)]
pub struct GetChainTip {
    pub height: u32,
    pub hash: BlockHash,
    /// The tip's blocks off the best chain.
    pub branchlen: u32,
    pub status: String,
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
    /// The node's estimate of its block and undo files' bytes.
    pub size_on_disk: u64,
}
