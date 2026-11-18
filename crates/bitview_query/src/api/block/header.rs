use bitcoin::{
    BlockHash as BitcoinBlockHash, block::Header as BitcoinHeader, consensus::Decodable,
    hashes::Hash, hex::DisplayHex,
};
use bitview_types::BlockHeader;
use brk_types::{BlkPosition, BlockHash, Height};
use vecdb::ReadableVec;

use crate::{Error, OptionData, Query, ReadPin, Result, check_bound};

pub(super) const HEADER_SIZE: usize = 80;

impl Query {
    /// Refuse a mismatched blocks directory before any stored positions are served.
    pub fn verify_block_positions(&self) -> Result<()> {
        let guard = self.pin_safe_lengths()?;
        if let Some(tip) = guard.lengths().last_height() {
            for height in [0, *tip / 4, *tip / 2, *tip - *tip / 4, *tip] {
                let height = Height::from(height);
                let hash = self.block_hash_by_height(height, &guard)?;
                self.verified_header(height, &hash, &guard)?;
            }
        }
        Ok(())
    }

    /// The header at `height`, checked against its pinned hash like every other header read.
    pub(crate) fn read_block_header_at(
        &self,
        height: Height,
        guard: &ReadPin,
    ) -> Result<BitcoinHeader> {
        let hash = self.block_hash_by_height(height, guard)?;
        let raw = self.verified_header(height, &hash, guard)?;
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

    /// Where the published block at `height` begins in the node's block files.
    pub(super) fn block_position(&self, height: Height, guard: &ReadPin) -> Result<BlkPosition> {
        check_bound(guard, height, guard.lengths().height)?;
        if height >= guard.lengths().height {
            return Err(Error::NotFound("Block not found".into()));
        }
        let position = self.indexer().vecs().blocks.position.collect_one(height);
        position.data()
    }

    /// The 80 header bytes of the published block at `height`, once they hash to `hash`: the one
    /// check that the indexed position holds the block asked for.
    pub(super) fn verified_header(
        &self,
        height: Height,
        hash: &BlockHash,
        guard: &ReadPin,
    ) -> Result<Vec<u8>> {
        let position = self.block_position(height, guard)?;
        let bytes = self.reader().read_raw_bytes(position, HEADER_SIZE)?;
        Self::verify_header(&bytes, hash).map_err(|error| {
            let found = BlockHash::from(BitcoinBlockHash::hash(&bytes));
            let path = self
                .blocks_dir()
                .join(format!("blk{:05}.dat", position.blk_index()));
            std::io::Error::other(format!(
                "{error} at height {height}: expected {hash}, found {found}, blk path {}",
                path.display()
            ))
        })?;
        Ok(bytes)
    }

    pub(super) fn block_header_hex_at_height(
        &self,
        height: Height,
        hash: &BlockHash,
        guard: &ReadPin,
    ) -> Result<String> {
        Ok(self
            .verified_header(height, hash, guard)?
            .to_lower_hex_string())
    }

    pub(crate) fn verify_header(bytes: &[u8], expected_hash: &BlockHash) -> Result<()> {
        debug_assert_eq!(bytes.len(), HEADER_SIZE);
        if BlockHash::from(BitcoinBlockHash::hash(bytes)) != *expected_hash {
            return Err(Error::Internal("Block header differs from index"));
        }
        Ok(())
    }
}
