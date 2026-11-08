use brk_types::{BlockHash, Height};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A stale block at a height of the best chain: a block of a fork the node knows (bitcoind's
/// `getchaintips`), as mempool.space lists it.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlockOrphan {
    pub height: Height,
    pub hash: BlockHash,
    /// Length of the fork's branch, in blocks
    #[schemars(example = 1)]
    pub branchlen: u32,
    pub status: BlockOrphanStatus,
    /// Hash of the block the stale block builds on
    pub prevhash: BlockHash,
}

/// How far the node validated a fork (bitcoind's `getchaintips` statuses for the forks it holds
/// as valid).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum BlockOrphanStatus {
    /// All of the branch's blocks are available, but they were never fully validated
    ValidHeaders,
    /// The branch is fully validated but not part of the best chain
    ValidFork,
}
