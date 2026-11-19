use bitcoin::Weight;
use brk_types::{BlkPosition, BlockHash, Height};
use vecdb::ReadableVec;

use super::header::HEADER_SIZE;
use crate::{Error, OptionData, Query, ReadPin, Result};

impl Query {
    /// The indexed size of the block at `height`, a height the caller knows is published.
    fn block_size(&self, height: Height) -> Result<usize> {
        let size = self.indexer().vecs().blocks.size.collect_one(height);
        let size = *size.data()? as usize;
        // Serialized bytes cannot exceed weight: a larger size is a corrupt column, not a block.
        debug_assert!(
            size as u64 <= Weight::MAX_BLOCK.to_wu(),
            "block {height}: {size} bytes"
        );
        Ok(size)
    }

    /// The block's bytes as the node stored them. The header's hash is the whole proof that the
    /// indexed position holds the block asked for. The transactions under it are the node's own
    /// record, served as stored: decoding them to recompute the merkle root, the witness
    /// commitment and the weight cost about 5 ms per MB on every request.
    pub(crate) fn block_raw_at_height(
        &self,
        height: Height,
        hash: &BlockHash,
        guard: &ReadPin,
    ) -> Result<Vec<u8>> {
        self.block_raw_with_position(height, hash, guard)
            .map(|(bytes, _)| bytes)
    }

    pub(crate) fn block_raw_with_position(
        &self,
        height: Height,
        hash: &BlockHash,
        guard: &ReadPin,
    ) -> Result<(Vec<u8>, BlkPosition)> {
        let position = self.block_position(height, guard)?;
        let bytes = self
            .reader()
            .read_raw_bytes(position, self.block_size(height)?)?;
        let header = bytes
            .first_chunk::<HEADER_SIZE>()
            .ok_or(Error::Internal("Indexed block smaller than a header"))?;
        Self::verify_header(header, hash)?;
        Ok((bytes, position))
    }

    /// The block's size, once its header is the block asked for: the transactions are not read.
    pub(super) fn block_raw_size_at_height(
        &self,
        height: Height,
        hash: &BlockHash,
        guard: &ReadPin,
    ) -> Result<u64> {
        self.verified_header(height, hash, guard)?;
        Ok(self.block_size(height)? as u64)
    }
}
