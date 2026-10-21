use brk_types::Height;

use crate::MerkleNode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Merkle inclusion proof for a transaction
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MerkleProof {
    pub block_height: Height,
    /// Merkle proof path, from the transaction's sibling up
    pub merkle: Vec<MerkleNode>,
    /// Transaction position in the block (0-indexed)
    #[schemars(example = 42)]
    pub pos: usize,
}
