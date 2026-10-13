use bitview_plugin_indexer::SafeLengths;
use bitview_types::{BlockInfo, BlockTxIndex};
use brk_types::{BlockHash, Height, Transaction, Txid};
use vecdb::ReadableVec;

use crate::{Error, OptionData, Query, ReadPin, Result};

/// An immutable block range pinned at the published safe lengths.
pub struct ResolvedBlocks {
    guard: SafeLengths,
    begin: usize,
    end: usize,
    anchor: Option<BlockHash>,
}

impl ResolvedBlocks {
    /// The final block in this range, not necessarily the current chain tip.
    pub fn anchor(&self) -> Option<BlockHash> {
        self.anchor
    }

    pub fn last_height(&self) -> Option<Height> {
        self.end.checked_sub(1).map(Height::from)
    }

    pub fn build(self, query: &Query) -> Result<Vec<BlockInfo>> {
        query.blocks_range_at(self.begin, self.end, &self.read_pin(query))
    }

    fn anchor_height(&self) -> Result<Height> {
        self.last_height()
            .ok_or_else(|| Error::NotFound("Block not found".into()))
    }

    fn anchor_pair(&self) -> Result<(Height, BlockHash)> {
        self.last_height()
            .zip(self.anchor)
            .ok_or_else(|| Error::NotFound("Block not found".into()))
    }

    /// Validate an in-block offset before HTTP conditional short-circuiting.
    pub fn validate_tx_index(&self, query: &Query, index: BlockTxIndex) -> Result<()> {
        let (_, count) = query.block_tx_range(self.anchor_height()?, &self.read_pin(query))?;
        if usize::from(index) >= count {
            return Err(Error::OutOfRange("Transaction index out of range".into()));
        }
        Ok(())
    }

    pub fn anchor_txids(self, query: &Query) -> Result<Vec<Txid>> {
        query.block_txids_by_height(self.anchor_height()?, &self.read_pin(query))
    }

    pub fn anchor_txid(self, query: &Query, index: BlockTxIndex) -> Result<Txid> {
        query.block_txid_at_index_by_height(
            self.anchor_height()?,
            index.into(),
            &self.read_pin(query),
        )
    }

    pub fn anchor_txs(
        self,
        query: &Query,
        start: BlockTxIndex,
        count: u32,
    ) -> Result<Vec<Transaction>> {
        query.block_txs_at_height(self.anchor_height()?, start, count, &self.read_pin(query))
    }

    /// Read the anchor's verified header while retaining the pin.
    /// Exact-hash callers resolve a single-row snapshot before using this.
    pub fn anchor_header_hex(self, query: &Query) -> Result<String> {
        let (height, hash) = self.anchor_pair()?;
        query.block_header_hex_at_height(height, &hash, &self.read_pin(query))
    }

    /// Read the anchor's raw block while retaining the pin.
    pub fn anchor_raw(self, query: &Query) -> Result<Vec<u8>> {
        let (height, hash) = self.anchor_pair()?;
        query.block_raw_at_height(height, &hash, &self.read_pin(query))
    }

    /// Return the framed raw length after verifying the anchor header.
    /// This does not read or validate transaction payload bytes.
    pub fn anchor_raw_size(self, query: &Query) -> Result<u64> {
        let (height, hash) = self.anchor_pair()?;
        query.block_raw_size_at_height(height, &hash, &self.read_pin(query))
    }

    pub(crate) fn range(&self) -> (usize, usize) {
        (self.begin, self.end)
    }

    /// Bounds this thread's reads by the blocks' pin while the returned pin lives.
    pub(crate) fn read_pin(&self, query: &Query) -> ReadPin {
        query.read_pin(self.guard.clone())
    }
}

impl Query {
    /// Check one caller-supplied height against the full canonical hash without
    /// consulting the prefix store. A miss falls back to worker resolution.
    pub fn try_resolve_block_snapshot(
        &self,
        hash: &BlockHash,
        height_hint: Height,
    ) -> Result<Option<ResolvedBlocks>> {
        let blocks = self.resolve_blocks(Some(height_hint), 1)?;
        if blocks.last_height() != Some(height_hint) || blocks.anchor() != Some(*hash) {
            return Ok(None);
        }
        Ok(Some(blocks))
    }

    /// Resolve an exact canonical hash and retain the pin until
    /// the selected row is consumed. The prefix-store lookup may perform I/O.
    pub fn resolve_block_snapshot(&self, hash: &BlockHash) -> Result<ResolvedBlocks> {
        let guard = self.pin_safe_lengths()?;
        let height = self.height_by_hash_at(hash, &guard)?;
        self.blocks_snapshot(Some(height), 1, guard)
    }

    pub fn resolve_blocks(
        &self,
        start_height: Option<Height>,
        count: u32,
    ) -> Result<ResolvedBlocks> {
        self.blocks_snapshot(start_height, count, self.pin_safe_lengths()?)
    }

    fn blocks_snapshot(
        &self,
        start_height: Option<Height>,
        count: u32,
        guard: ReadPin,
    ) -> Result<ResolvedBlocks> {
        let (begin, end) = Self::resolve_block_range(start_height, count, guard.lengths().height);
        let anchor = end
            .checked_sub(1)
            .map(|height| {
                self.indexer()
                    .vecs()
                    .blocks
                    .blockhash
                    .collect_one_at(height)
                    .data()
            })
            .transpose()?;
        Ok(ResolvedBlocks {
            guard: guard.into_inner(),
            begin,
            end,
            anchor,
        })
    }
}
