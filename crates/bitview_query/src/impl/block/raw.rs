use std::io::Read;

use bitcoin::{Block as BitcoinBlock, Weight, consensus::deserialize, p2p::Magic};
use bitview_plugin_indexer::SafeLengths;
use brk_reader::BlkRead;
use brk_types::{BlkPosition, BlockHash, Height};
use vecdb::ReadableVec;

use crate::{Error, OptionData, Query, Result};

// Serialized bytes cannot exceed weight: each byte contributes at least one WU.
const MAX_BLOCK_BYTES: u64 = Weight::MAX_BLOCK.to_wu();

struct RawRecord {
    reader: BlkRead,
    size: u64,
    weight: u64,
    count: u32,
}

impl Query {
    fn open_raw_record(&self, height: Height, guard: &SafeLengths) -> Result<RawRecord> {
        let safe = guard.lengths();
        if height >= safe.height {
            return Err(Error::NotFound("Block not found".into()));
        }
        let vecs = self.indexer().vecs();
        let position = vecs.blocks.position.collect_one(height).data()?;
        let size = *vecs.blocks.total.collect_one(height).data()?;
        let weight = u64::from(*vecs.blocks.weight.collect_one(height).data()?);
        if weight > Weight::MAX_BLOCK.to_wu() || size > weight {
            return Err(Error::Internal("Invalid indexed block weight"));
        }
        let first = vecs
            .transactions
            .first_tx_index
            .collect_one(height)
            .data()?;
        let next_height = usize::from(height) + 1;
        let next = if next_height < usize::from(safe.height) {
            vecs.transactions
                .first_tx_index
                .collect_one_at(next_height)
                .data()?
        } else {
            safe.tx_index
        };
        let count = Self::block_tx_count(first, next, safe.tx_index)?;
        let offset = position
            .offset()
            .checked_sub(8)
            .ok_or(Error::Internal("Block position precedes its frame"))?;
        let reader = self
            .reader()
            .reader_at(BlkPosition::new(position.blk_index(), offset))?;
        Ok(RawRecord {
            reader,
            size,
            weight,
            count,
        })
    }

    fn read_raw_record(mut reader: impl Read, size: u64, hash: &BlockHash) -> Result<Vec<u8>> {
        let header = Self::read_raw_prefix(&mut reader, size, hash)?;
        let mut bytes = vec![0; size as usize];
        bytes[..80].copy_from_slice(&header);
        reader.read_exact(&mut bytes[80..])?;
        Ok(bytes)
    }

    fn read_raw_prefix(mut reader: impl Read, size: u64, hash: &BlockHash) -> Result<[u8; 80]> {
        if !(81..=MAX_BLOCK_BYTES).contains(&size) {
            return Err(Error::Internal("Invalid indexed block size"));
        }
        // This reader indexes Bitcoin-mainnet blk records (same magic as scan).
        // Read framing and header together, before allocating the payload.
        let mut prefix = [0u8; 88];
        reader.read_exact(&mut prefix)?;
        if prefix[..4] != Magic::BITCOIN.to_bytes()
            || u64::from(u32::from_le_bytes(prefix[4..8].try_into().unwrap())) != size
        {
            return Err(Error::Internal("Block frame differs from index"));
        }
        Self::verify_header(&prefix[8..], hash)?;
        Ok(prefix[8..].try_into().unwrap())
    }

    fn verify_raw_payload(bytes: &[u8], weight: u64, count: u32) -> Result<()> {
        // Check the indexed count before a decoder allocates its transaction list.
        let transactions = bytes
            .get(80..)
            .ok_or(Error::Internal("Raw block shorter than header"))?;
        Self::read_block_tx_count(transactions, count)?;
        let block: BitcoinBlock =
            deserialize(bytes).map_err(|_| Error::Internal("Invalid raw block payload"))?;
        if weight > Weight::MAX_BLOCK.to_wu()
            || block.weight().to_wu() != weight
            || !block.check_merkle_root()
            || !block.check_witness_commitment()
        {
            return Err(Error::Internal("Raw block commitments differ from index"));
        }
        Ok(())
    }

    /// Borrow the pinned immutable prefix through the complete payload read.
    pub(super) fn block_raw_at_height(
        &self,
        height: Height,
        hash: &BlockHash,
        guard: &SafeLengths,
    ) -> Result<Vec<u8>> {
        let mut record = self.open_raw_record(height, guard)?;
        let bytes = Self::read_raw_record(&mut record.reader, record.size, hash)?;
        Self::verify_raw_payload(&bytes, record.weight, record.count)?;
        Ok(bytes)
    }
    /// Validate framing and identity without reading or allocating the payload.
    /// Borrows the pinned prefix; payload integrity remains a GET check.
    pub(super) fn block_raw_size_at_height(
        &self,
        height: Height,
        hash: &BlockHash,
        guard: &SafeLengths,
    ) -> Result<u64> {
        let mut record = self.open_raw_record(height, guard)?;
        Self::read_raw_prefix(&mut record.reader, record.size, hash)?;
        Ok(record.size)
    }
}
