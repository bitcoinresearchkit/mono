use bitview_plugin::PublicationReadGuard;
use bitview_primitives::PoolSlug;
use bitview_types::BlockInfoV1;
use brk_error::{Error, OptionData, Result};
use brk_types::{BlockHash, Dollars, Height};
use vecdb::ReadableVec;

use crate::{Query, ResolvedBlocks};

/// A pool-block page resolved against one exact published chain view.
pub struct ResolvedPoolBlocks {
    _publication: PublicationReadGuard,
    chain: ResolvedBlocks,
    heights: Vec<Height>,
    prices: Vec<Dollars>,
    activity_anchor: Option<BlockHash>,
}

impl ResolvedPoolBlocks {
    #[inline]
    pub const fn activity_anchor(&self) -> Option<BlockHash> {
        self.activity_anchor
    }

    pub fn heights(&self) -> &[Height] {
        &self.heights
    }

    /// Captured prices in the same descending order as the selected heights.
    pub fn prices(&self) -> &[Dollars] {
        &self.prices
    }
}

impl Query {
    /// Resolve the page's exact block heights and activity anchor once.
    pub fn resolve_pool_blocks(
        &self,
        slug: PoolSlug,
        before_height: Option<Height>,
        limit: usize,
    ) -> Result<ResolvedPoolBlocks> {
        let publication = self.read_publication()?;
        let chain = self.resolve_blocks(None, 0)?;
        let tip = chain.last_height().ok_or(Error::StateUpdating)?;
        let through_height = before_height.unwrap_or(tip).min(tip);
        let heights = self
            .plugins()
            .pools
            .heights
            .latest_heights(slug, through_height, limit);
        let activity_anchor = heights
            .first()
            .map(|height| {
                self.indexer()
                    .vecs()
                    .blocks
                    .blockhash
                    .collect_one(*height)
                    .data()
            })
            .transpose()?;
        let prices = heights
            .iter()
            .map(|height| {
                self.plugins()
                    .price
                    .spot
                    .cents
                    .height
                    .collect_one(*height)
                    .data()
                    .map(Dollars::from)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(ResolvedPoolBlocks {
            _publication: publication,
            chain,
            heights,
            prices,
            activity_anchor,
        })
    }

    /// Load a resolved page without repeating its pool-height lookup.
    pub fn pool_blocks_resolved(&self, resolved: ResolvedPoolBlocks) -> Result<Vec<BlockInfoV1>> {
        let ResolvedPoolBlocks {
            _publication,
            chain,
            heights,
            prices,
            ..
        } = resolved;
        chain.build_v1_heights(self, &heights, &prices, _publication)
    }
}
