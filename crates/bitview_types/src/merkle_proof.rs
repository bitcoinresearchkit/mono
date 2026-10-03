use brk_types::Height;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Merkle inclusion proof for a transaction
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MerkleProof {
    /// Block height containing the transaction
    pub block_height: Height,
    /// Merkle proof path (hex-encoded hashes)
    pub merkle: Vec<String>,
    /// Transaction position in the block (0-indexed)
    #[schemars(example = 42)]
    pub pos: usize,
}
