use brk_error::{Error, Result};
use brk_types::{BlockHash, BlockHashPrefix, Height};

use crate::Query;

/// An exact block hash resolved to its current best-chain height.
///
/// The fields stay private so callers cannot construct a stale or mismatched
/// hash-height pair. Query methods revalidate the pair after an async handoff.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedBlock {
    hash: BlockHash,
    height: Height,
}

impl ResolvedBlock {
    /// The block's best-chain height when it was resolved.
    #[inline]
    pub const fn height(self) -> Height {
        self.height
    }
}

impl Query {
    /// Resolve an exact best-chain block hash once.
    pub fn resolve_block(&self, hash: &BlockHash) -> Result<ResolvedBlock> {
        let height = self
            .indexer()
            .stores()
            .block_height(&BlockHashPrefix::from(hash))?
            .ok_or(Error::NotFound("Block not found".into()))?;

        self.validate_block_at_height(hash, height)?;

        Ok(ResolvedBlock {
            hash: *hash,
            height,
        })
    }

    /// Hash to height, requiring an exact best-chain match at the safe bound.
    #[inline]
    pub fn height_by_hash(&self, hash: &BlockHash) -> Result<Height> {
        self.resolve_block(hash).map(ResolvedBlock::height)
    }

    /// Revalidate a resolved pair without repeating the prefix-store lookup.
    /// This protects server calls from a reorg between cache selection and the
    /// blocking response task.
    pub(super) fn revalidate_block(&self, block: ResolvedBlock) -> Result<Height> {
        self.validate_block_at_height(&block.hash, block.height)?;
        Ok(block.height)
    }

    /// Validate between two safe-bound snapshots. Rollback lowers the bound
    /// before mutating vectors, so either snapshot rejects a concurrent change.
    pub(crate) fn validate_block_at_height(&self, hash: &BlockHash, height: Height) -> Result<()> {
        if height >= self.safe_lengths().height {
            return Err(Error::NotFound("Block not found".into()));
        }

        if self.indexer().vecs().blocks.blockhash.get(height) != Some(*hash) {
            return Err(Error::NotFound("Block not found".into()));
        }

        if height >= self.safe_lengths().height {
            return Err(Error::NotFound("Block not found".into()));
        }

        Ok(())
    }
}
