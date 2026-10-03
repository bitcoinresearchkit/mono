use bitcoin::block::Header;
use brk_types::BlockHash;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Block header matching mempool.space's format.
/// Contains the same fields as bitcoin::block::Header
/// but serialized for the JSON API.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlockHeader {
    /// Block version, used for soft fork signaling
    pub version: u32,

    /// Previous block hash
    #[serde(rename = "previousblockhash")]
    pub previous_block_hash: BlockHash,

    /// Merkle root of the transaction tree
    pub merkle_root: String,

    /// Compact target (bits)
    pub bits: u32,

    /// Nonce used to produce a valid block hash
    pub nonce: u32,
}

impl From<Header> for BlockHeader {
    fn from(h: Header) -> Self {
        Self {
            version: h.version.to_consensus() as u32,
            previous_block_hash: BlockHash::from(h.prev_blockhash),
            merkle_root: h.merkle_root.to_string(),
            bits: h.bits.to_consensus(),
            nonce: h.nonce,
        }
    }
}
