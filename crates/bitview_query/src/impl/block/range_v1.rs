use bitview_plugin::PublicationReadGuard;
use bitview_types::BlockInfoV1;
use brk_types::{BlockHash, Dollars, Height};
use vecdb::{ReadableVec, VecIndex};

use super::ResolvedBlocks;
use crate::{Error, Query, Result};

/// Blocks and their published prices held in one stable chain view.
pub struct ResolvedBlocksV1 {
    _publication: PublicationReadGuard,
    blocks: ResolvedBlocks,
    prices: Vec<Dollars>,
}

impl ResolvedBlocksV1 {
    fn new(
        blocks: ResolvedBlocks,
        prices: Vec<Dollars>,
        publication: PublicationReadGuard,
    ) -> Result<Self> {
        let (begin, end, _) = blocks.range();
        if prices.len() != end - begin {
            return Err(Error::Internal("Incomplete block prices"));
        }
        Ok(Self {
            blocks,
            prices,
            _publication: publication,
        })
    }

    pub fn anchor(&self) -> Option<BlockHash> {
        self.blocks.anchor()
    }

    pub fn last_height(&self) -> Option<Height> {
        self.blocks.last_height()
    }

    /// Prices in ascending height order, reused when building descending rows.
    pub fn prices(&self) -> &[Dollars] {
        &self.prices
    }

    pub fn build(self, query: &Query) -> Result<Vec<BlockInfoV1>> {
        let (begin, end, lengths) = self.blocks.range();
        let build = query.capture_blocks_v1_range(begin, end, lengths, Some(self.prices))?;
        drop(self._publication);
        drop(self.blocks);
        build()
    }
}

impl Query {
    /// Resolve using only already-materialized prices; otherwise retry on a worker.
    pub fn try_resolve_blocks_v1(
        &self,
        start_height: Option<Height>,
        count: u32,
    ) -> Result<Option<ResolvedBlocksV1>> {
        let Some(publication) = self.try_read_publication() else {
            return Ok(None);
        };
        let Some(blocks) = self.try_resolve_blocks(start_height, count)? else {
            return Ok(None);
        };
        self.try_block_prices(blocks, publication)
    }

    /// Validate a caller's height hint without performing a hash-store lookup.
    pub fn try_resolve_block_v1(
        &self,
        hash: &BlockHash,
        height_hint: Height,
    ) -> Result<Option<ResolvedBlocksV1>> {
        let Some(publication) = self.try_read_publication() else {
            return Ok(None);
        };
        let Some(blocks) = self.try_resolve_block_snapshot(hash, height_hint)? else {
            return Ok(None);
        };
        self.try_block_prices(blocks, publication)
    }

    fn try_block_prices(
        &self,
        blocks: ResolvedBlocks,
        publication: PublicationReadGuard,
    ) -> Result<Option<ResolvedBlocksV1>> {
        let (begin, end, _) = blocks.range();
        let prices =
            if begin == end {
                Vec::new()
            } else {
                let mut prices = Vec::with_capacity(end - begin);
                if !self.plugins().price.spot.cents.height.read_cached_into_at(
                    begin,
                    end,
                    &mut prices,
                ) {
                    return Ok(None);
                }
                prices.into_iter().map(Dollars::from).collect()
            };
        ResolvedBlocksV1::new(blocks, prices, publication).map(Some)
    }

    /// Resolve on a worker without materializing or retaining the price history.
    pub fn resolve_blocks_v1(
        &self,
        start_height: Option<Height>,
        count: u32,
    ) -> Result<ResolvedBlocksV1> {
        let publication = self.read_publication()?;
        let blocks = self.resolve_blocks(start_height, count)?;
        self.block_prices(blocks, publication)
    }

    /// Resolve the exact canonical hash on a worker under one publication guard.
    pub fn resolve_block_v1(&self, hash: &BlockHash) -> Result<ResolvedBlocksV1> {
        let publication = self.read_publication()?;
        let blocks = self.resolve_block_snapshot(hash)?;
        self.block_prices(blocks, publication)
    }

    fn block_prices(
        &self,
        blocks: ResolvedBlocks,
        publication: PublicationReadGuard,
    ) -> Result<ResolvedBlocksV1> {
        let (begin, end, _) = blocks.range();
        let mut prices = Vec::with_capacity(end - begin);
        self.plugins()
            .price
            .spot
            .cents
            .height
            .for_each_range_dyn_at(begin, end, &mut |cents| prices.push(Dollars::from(cents)));
        ResolvedBlocksV1::new(blocks, prices, publication)
    }
}

impl ResolvedBlocks {
    /// Capture sparse descending V1 rows, then build them without read guards.
    /// Adjacent heights share a bulk read; supplied prices are never re-read.
    pub(crate) fn build_v1_heights(
        self,
        query: &Query,
        heights: &[Height],
        prices: &[Dollars],
        publication: PublicationReadGuard,
    ) -> Result<Vec<BlockInfoV1>> {
        let (_, _, lengths) = self.range();
        if heights.len() != prices.len()
            || heights.iter().any(|height| *height >= lengths.height)
            || heights.windows(2).any(|pair| pair[0] <= pair[1])
        {
            return Err(Error::Internal("Invalid sparse block selection"));
        }
        let mut builders = Vec::new();
        let mut begin = 0;
        while begin < heights.len() {
            let mut end = begin + 1;
            while end < heights.len() && heights[end].to_usize() + 1 == heights[end - 1].to_usize()
            {
                end += 1;
            }
            let ascending_prices = prices[begin..end].iter().rev().copied().collect();
            builders.push(query.capture_blocks_v1_range(
                heights[end - 1].to_usize(),
                heights[begin].to_usize() + 1,
                lengths,
                Some(ascending_prices),
            )?);
            begin = end;
        }
        drop(publication);
        drop(self);
        let mut blocks = Vec::with_capacity(heights.len());
        for build in builders {
            blocks.extend(build()?);
        }
        Ok(blocks)
    }
}
