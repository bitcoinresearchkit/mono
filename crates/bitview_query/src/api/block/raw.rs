use brk_types::{BlkPosition, BlockHash, Height};
use vecdb::ReadableVec;

use super::header::HEADER_SIZE;
use crate::{Error, OptionData, Query, ReadPin, Result};

impl Query {
    /// Where the published block at `height` lies in the node's block files, and its size.
    fn raw_record(&self, height: Height, guard: &ReadPin) -> Result<(BlkPosition, usize)> {
        let position = self.block_position(height, guard)?;
        let size = self.indexer().vecs().blocks.size.collect_one(height);
        Ok((position, *size.data()? as usize))
    }

    /// The block's bytes as the node stored them. The header's hash is the whole proof that the
    /// indexed position holds the block asked for. The transactions under it are the node's own
    /// record, served as stored: decoding them to recompute the merkle root, the witness
    /// commitment and the weight cost about 20 ms per MB on every request.
    pub(super) fn block_raw_at_height(
        &self,
        height: Height,
        hash: &BlockHash,
        guard: &ReadPin,
    ) -> Result<Vec<u8>> {
        let (position, size) = self.raw_record(height, guard)?;
        let bytes = self.reader().read_raw_bytes(position, size)?;
        let header = bytes
            .first_chunk::<HEADER_SIZE>()
            .ok_or(Error::Internal("Indexed block smaller than a header"))?;
        Self::verify_header(header, hash)?;
        Ok(bytes)
    }

    /// The block's size, once its header is the block asked for: the transactions are not read.
    pub(super) fn block_raw_size_at_height(
        &self,
        height: Height,
        hash: &BlockHash,
        guard: &ReadPin,
    ) -> Result<u64> {
        let (position, size) = self.raw_record(height, guard)?;
        Self::verify_header(&self.reader().read_raw_bytes(position, HEADER_SIZE)?, hash)?;
        Ok(size as u64)
    }
}
