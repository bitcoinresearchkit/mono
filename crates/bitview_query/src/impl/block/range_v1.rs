use bitview_types::BlockInfoV1;
use brk_types::{BlockHash, Dollars, Height};
use vecdb::{ReadableVec, VecIndex};

use super::ResolvedBlocks;
use crate::{Error, Query, Result};

/// Blocks and their published prices, held by the blocks' pin.
pub struct ResolvedBlocksV1 {
    blocks: ResolvedBlocks,
    prices: Vec<Dollars>,
}

impl ResolvedBlocksV1 {
    fn new(blocks: ResolvedBlocks, prices: Vec<Dollars>) -> Result<Self> {
        let (begin, end) = blocks.range();
        if prices.len() != end - begin {
            return Err(Error::Internal("Incomplete block prices"));
        }
        Ok(Self { blocks, prices })
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
        let (begin, end) = self.blocks.range();
        let pin = self.blocks.read_pin(query);
        let build = query.capture_blocks_v1_range(begin, end, &pin, Some(self.prices))?;
        drop(pin);
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
        self.try_block_prices(self.blocks_window(start_height, count)?)
    }

    /// Validate a caller's height hint without performing a hash-store lookup.
    pub fn try_resolve_block_v1(
        &self,
        hash: &BlockHash,
        height_hint: Height,
    ) -> Result<Option<ResolvedBlocksV1>> {
        let Some(blocks) = self.try_resolve_block_snapshot(hash, height_hint)? else {
            return Ok(None);
        };
        self.try_block_prices(blocks)
    }

    fn try_block_prices(&self, blocks: ResolvedBlocks) -> Result<Option<ResolvedBlocksV1>> {
        let (begin, end) = blocks.range();
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
        ResolvedBlocksV1::new(blocks, prices).map(Some)
    }

    /// Resolve on a worker without materializing or retaining the price history.
    pub fn resolve_blocks_v1(
        &self,
        start_height: Option<Height>,
        count: u32,
    ) -> Result<ResolvedBlocksV1> {
        let blocks = self.blocks_window(start_height, count)?;
        self.block_prices(blocks)
    }

    /// Resolve the exact canonical hash on a worker.
    pub fn resolve_block_v1(&self, hash: &BlockHash) -> Result<ResolvedBlocksV1> {
        let blocks = self.resolve_block_snapshot(hash)?;
        self.block_prices(blocks)
    }

    fn block_prices(&self, blocks: ResolvedBlocks) -> Result<ResolvedBlocksV1> {
        let (begin, end) = blocks.range();
        let mut prices = Vec::with_capacity(end - begin);
        self.plugins()
            .price
            .spot
            .cents
            .height
            .for_each_range_dyn_at(begin, end, &mut |cents| prices.push(Dollars::from(cents)));
        ResolvedBlocksV1::new(blocks, prices)
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
    ) -> Result<Vec<BlockInfoV1>> {
        let pin = self.read_pin(query);
        if heights.len() != prices.len()
            || heights.iter().any(|height| *height >= pin.lengths().height)
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
                &pin,
                Some(ascending_prices),
            )?);
            begin = end;
        }
        drop(pin);
        drop(self);
        let mut blocks = Vec::with_capacity(heights.len());
        for build in builders {
            blocks.extend(build()?);
        }
        Ok(blocks)
    }
}
