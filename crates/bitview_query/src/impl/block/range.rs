use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::Lengths;
use bitview_types::{BlockInfo, BlockTxIndex};
use brk_types::{BlockHash, Height, Transaction, Txid};
use vecdb::ReadableVec;

use crate::{Error, OptionData, Query, Result};

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
        query.blocks_range_at(self.begin, self.end, &self.guard)
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
        let (_, count) = query.block_tx_range(self.anchor_height()?, &self.guard)?;
        if usize::from(index) >= count {
            return Err(Error::OutOfRange("Transaction index out of range".into()));
        }
        Ok(())
    }

    pub fn anchor_txids(self, query: &Query) -> Result<Vec<Txid>> {
        query.block_txids_by_height(self.anchor_height()?, &self.guard)
    }

    pub fn anchor_txid(self, query: &Query, index: BlockTxIndex) -> Result<Txid> {
        query.block_txid_at_index_by_height(self.anchor_height()?, index.into(), &self.guard)
    }

    pub fn anchor_txs(
        self,
        query: &Query,
        start: BlockTxIndex,
        count: u32,
    ) -> Result<Vec<Transaction>> {
        query.block_txs_at_height(self.anchor_height()?, start, count, &self.guard)
    }

    /// Read the anchor's verified header while retaining publication exclusion.
    /// Exact-hash callers resolve a single-row snapshot before using this.
    pub fn anchor_header_hex(self, query: &Query) -> Result<String> {
        let (height, hash) = self.anchor_pair()?;
        query.block_header_hex_at_height(height, &hash, &self.guard)
    }

    /// Read the anchor's raw block while retaining publication exclusion.
    pub fn anchor_raw(self, query: &Query) -> Result<Vec<u8>> {
        let (height, hash) = self.anchor_pair()?;
        query.block_raw_at_height(height, &hash, &self.guard)
    }

    /// Return the framed raw length after verifying the anchor header.
    /// This does not read or validate transaction payload bytes.
    pub fn anchor_raw_size(self, query: &Query) -> Result<u64> {
        let (height, hash) = self.anchor_pair()?;
        query.block_raw_size_at_height(height, &hash, &self.guard)
    }

    pub(crate) fn range(&self) -> (usize, usize, Lengths) {
        (self.begin, self.end, self.guard.lengths())
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
        let Some(blocks) = self.try_resolve_blocks(Some(height_hint), 1)? else {
            return Ok(None);
        };
        if blocks.last_height() != Some(height_hint) || blocks.anchor() != Some(*hash) {
            return Ok(None);
        }
        Ok(Some(blocks))
    }

    /// Resolve an exact canonical hash and retain publication exclusion until
    /// the selected row is consumed. The prefix-store lookup may perform I/O.
    pub fn resolve_block_snapshot(&self, hash: &BlockHash) -> Result<ResolvedBlocks> {
        let guard = self.pin_safe_lengths()?;
        let height = self.height_by_hash_at(hash, &guard)?;
        self.blocks_snapshot(Some(height), 1, guard)
    }

    /// Returns `None` during publication; callers may retry on a blocking worker.
    pub fn try_resolve_blocks(
        &self,
        start_height: Option<Height>,
        count: u32,
    ) -> Result<Option<ResolvedBlocks>> {
        self.indexer()
            .try_pin_safe_lengths()
            .map(|guard| self.blocks_snapshot(start_height, count, guard))
            .transpose()
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
        guard: SafeLengths,
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
            guard,
            begin,
            end,
            anchor,
        })
    }
}
