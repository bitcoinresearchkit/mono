use crate::ReadPin;
use bitview_types::BlockStatus;
use brk_types::{BlockHash, Height};
use vecdb::ReadableVec;

use crate::{Error, OptionData, Query, Result};

impl Query {
    /// As Esplora (mempool.space's backend) answers: a block outside the published best chain,
    /// orphaned or never seen, is `in_best_chain: false`, not missing.
    pub fn block_status(&self, hash: &BlockHash) -> Result<BlockStatus> {
        let guard = self.pin_safe_lengths()?;
        match self.height_by_hash_at(hash, &guard) {
            Ok(height) => self.block_status_at_height(height, &guard),
            Err(Error::NotFound(_)) => Ok(BlockStatus {
                in_best_chain: false,
                height: None,
                next_best: None,
            }),
            Err(error) => Err(error),
        }
    }

    fn block_status_at_height(&self, height: Height, guard: &ReadPin) -> Result<BlockStatus> {
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
