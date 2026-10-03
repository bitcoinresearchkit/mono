use bitview_plugin_indexer::SafeLengths;
use bitview_types::BlockStatus;
use brk_types::{BlockHash, Height};
use vecdb::ReadableVec;

use crate::{OptionData, Query, Result};

impl Query {
    pub fn block_status(&self, hash: &BlockHash) -> Result<BlockStatus> {
        let guard = self.pin_safe_lengths()?;
        let height = self.height_by_hash_at(hash, &guard)?;
        self.block_status_at_height(height, &guard)
    }

    fn block_status_at_height(&self, height: Height, guard: &SafeLengths) -> Result<BlockStatus> {
        let tip = guard.lengths().last_height().data()?;
        let next_best = if height < tip {
            Some(
                self.indexer()
                    .vecs()
                    .blocks
                    .blockhash
                    .collect_one(height.incremented())
                    .data()?,
            )
        } else {
            None
        };

        Ok(BlockStatus::in_best_chain(height, next_best))
    }
}
