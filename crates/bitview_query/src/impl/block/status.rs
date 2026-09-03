use brk_error::{OptionData, Result};
use brk_types::{BlockHash, BlockStatus, Height};

use super::ResolvedBlock;
use crate::Query;

impl Query {
    pub fn block_status(&self, hash: &BlockHash) -> Result<BlockStatus> {
        let block = self.resolve_block(hash)?;
        self.block_status_at_height(block.height())
    }

    /// Status for a block previously resolved by exact hash. Returns
    /// `NotFound` if the block was displaced before this read.
    pub fn block_status_resolved(&self, block: ResolvedBlock) -> Result<BlockStatus> {
        let height = self.revalidate_block(block)?;
        self.block_status_at_height(height)
    }

    fn block_status_at_height(&self, height: Height) -> Result<BlockStatus> {
        let tip = self.height();
        let next_best = if height < tip {
            Some(
                self.indexer()
                    .vecs()
                    .blocks
                    .blockhash
                    .get(height.incremented())
                    .data()?,
            )
        } else {
            None
        };

        Ok(BlockStatus::in_best_chain(height, next_best))
    }
}
