use crate::{ReadPin, check_bound};
use bitview_primitives::BlockHashPrefix;
use brk_types::{BlockHash, Height};
use vecdb::ReadableVec;

use crate::{Error, Query, Result};

pub mod info;
pub mod range;
pub mod range_v1;
pub mod raw;
pub mod status;
pub mod timestamp;
pub mod txs;

pub use range::ResolvedBlocks;
pub use range_v1::ResolvedBlocksV1;
pub use timestamp::ResolvedBlockTimestamp;

impl Query {
    fn height_by_hash_at(&self, hash: &BlockHash, guard: &ReadPin) -> Result<Height> {
        let height = self
            .indexer()
            .stores()
            .block_height(&BlockHashPrefix::from(hash))?
            .ok_or_else(|| Error::NotFound("Block not found".into()))?;
        self.validate_block_at_height(hash, height, guard)?;
        Ok(height)
    }

    pub(crate) fn validate_block_at_height(
        &self,
        hash: &BlockHash,
        height: Height,
        guard: &ReadPin,
    ) -> Result<()> {
        check_bound(guard, height, guard.lengths().height)?;
        if height >= guard.lengths().height {
            return Err(Error::NotFound("Block not found".into()));
        }

        // Validate one stored row without materializing the hash-history cache.
        if self.indexer().vecs().blocks.blockhash.collect_one(height) != Some(*hash) {
            return Err(Error::NotFound("Block not found".into()));
        }

        Ok(())
    }
}
