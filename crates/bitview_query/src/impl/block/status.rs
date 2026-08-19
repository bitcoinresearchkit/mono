use brk_error::OptionData;
use brk_types::{BlockHash, BlockStatus, Height};

use crate::Query;

impl Query {
    pub fn block_status(&self, hash: &BlockHash) -> brk_error::Result<BlockStatus> {
        let height = self.height_by_hash(hash)?;
        self.block_status_by_height(height)
    }

    fn block_status_by_height(&self, height: Height) -> brk_error::Result<BlockStatus> {
        let bound = self.safe_lengths().height;

        if height >= bound {
            return Ok(BlockStatus::not_in_best_chain());
        }

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
