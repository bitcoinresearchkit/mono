use brk_error::{OptionData, Result};
use brk_types::{BlockHash, Height};
use vecdb::ReadableVec;

use super::ResolvedBlock;
use crate::Query;

impl Query {
    pub fn block_raw(&self, hash: &BlockHash) -> Result<Vec<u8>> {
        let block = self.resolve_block(hash)?;
        self.block_raw_at_height(block.height())
    }

    /// Raw bytes for a block previously resolved by exact hash. Returns
    /// `NotFound` if the block was displaced before this read.
    pub fn block_raw_resolved(&self, block: ResolvedBlock) -> Result<Vec<u8>> {
        let height = self.revalidate_block(block)?;
        self.block_raw_at_height(height)
    }

    fn block_raw_at_height(&self, height: Height) -> Result<Vec<u8>> {
        let indexer = self.indexer();
        let position = indexer.vecs().blocks.position.collect_one(height).data()?;
        let size = indexer.vecs().blocks.total.collect_one(height).data()?;

        self.reader().read_raw_bytes(position, *size as usize)
    }
}
