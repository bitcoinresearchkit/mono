use bitcoin::{
    BlockHash as BitcoinBlockHash, block::Header as BitcoinHeader, consensus::Decodable,
    hashes::Hash, hex::DisplayHex,
};
use bitview_types::BlockHeader;
use brk_types::{BlockHash, Height};
use vecdb::ReadableVec;

use crate::{Error, OptionData, Query, ReadPin, Result, check_bound};

pub(super) const HEADER_SIZE: usize = 80;

impl Query {
    /// The header at `height`, checked against its pinned hash like every other header read.
    pub(crate) fn read_block_header_at(
        &self,
        height: Height,
        guard: &ReadPin,
    ) -> Result<BitcoinHeader> {
        let hash = self.block_hash_by_height(height, guard)?;
        let position = self
            .indexer()
            .vecs()
            .blocks
            .position
            .collect_one(height)
            .data()?;
        let raw = self.reader().read_raw_bytes(position, HEADER_SIZE)?;
        Self::verify_header(&raw, &hash)?;
        BitcoinHeader::consensus_decode(&mut raw.as_slice())
            .map_err(|_| Error::Internal("Failed to decode block header"))
    }

    /// Decode exactly one header and bind its fields to the indexed block hash.
    /// Successful decoding alone does not detect a wrong blk-file position or
    /// changed header bytes. Fail before returning or parsing dependent data.
    pub(super) fn decode_header(bytes: &[u8], expected_hash: &BlockHash) -> Result<BlockHeader> {
        Self::verify_header(bytes, expected_hash)?;
        let raw = BitcoinHeader::consensus_decode(&mut &bytes[..])
            .map_err(|_| Error::Internal("Failed to decode block header"))?;
        Ok(BlockHeader::from(raw))
    }

    pub(super) fn block_header_hex_at_height(
        &self,
        height: Height,
        hash: &BlockHash,
        guard: &ReadPin,
    ) -> Result<String> {
        check_bound(guard, height, guard.lengths().height)?;
        if height >= guard.lengths().height {
            return Err(Error::NotFound("Block not found".into()));
        }
        let position = self
            .indexer()
            .vecs()
            .blocks
            .position
            .collect_one(height)
            .data()?;
        let bytes = self.reader().read_raw_bytes(position, HEADER_SIZE)?;
        Self::verify_header(&bytes, hash)?;
        Ok(bytes.to_lower_hex_string())
    }

    pub(crate) fn verify_header(bytes: &[u8], expected_hash: &BlockHash) -> Result<()> {
        debug_assert_eq!(bytes.len(), HEADER_SIZE);
        if BlockHash::from(BitcoinBlockHash::hash(bytes)) != *expected_hash {
            return Err(Error::Internal("Block header differs from index"));
        }
        Ok(())
    }
}
