use brk_error::Result;
use brk_types::{BlockHash, BlockInfoV1, Height, PoolSlug};
use vecdb::VecIndex;

use crate::Query;

/// A pool-block page resolved against one exact published chain view.
pub struct ResolvedPoolBlocks {
    heights: Vec<Height>,
    activity_anchor: BlockHash,
}

impl ResolvedPoolBlocks {
    #[inline]
    pub const fn activity_anchor(&self) -> BlockHash {
        self.activity_anchor
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
        let tip = self.height();
        let through_height = before_height.unwrap_or(tip).min(tip);
        let heights = self
            .plugins()
            .pools
            .heights
            .latest_heights(slug, through_height, limit);
        let activity_anchor = match heights.first() {
            Some(height) => self.block_hash_by_height(*height)?,
            None => self.tip_blockhash(),
        };

        Ok(ResolvedPoolBlocks {
            heights,
            activity_anchor,
        })
    }

    /// Load a resolved page without repeating its pool-height lookup.
    pub fn pool_blocks_resolved(&self, resolved: ResolvedPoolBlocks) -> Result<Vec<BlockInfoV1>> {
        let ResolvedPoolBlocks {
            heights,
            activity_anchor,
        } = resolved;
        let anchor_height = heights.first().copied();

        if let Some(height) = anchor_height {
            self.validate_block_at_height(&activity_anchor, height)?;
        }

        let mut blocks = Vec::with_capacity(heights.len());
        let mut i = 0;
        while i < heights.len() {
            let hi = heights[i].to_usize();
            while i + 1 < heights.len() && heights[i + 1].to_usize() + 1 == heights[i].to_usize() {
                i += 1;
            }
            let mut range =
                crate::r#impl::block::blocks_v1_range(self, heights[i].to_usize(), hi + 1)?;
            blocks.append(&mut range);
            i += 1;
        }

        if let Some(height) = anchor_height {
            self.validate_block_at_height(&activity_anchor, height)?;
        }

        Ok(blocks)
    }

    /// Page of blocks mined by `slug`, in descending height order, capped at
    /// `limit`. `before_height` is the inclusive upper bound to paginate from.
    pub fn pool_blocks(
        &self,
        slug: PoolSlug,
        before_height: Option<Height>,
        limit: usize,
    ) -> Result<Vec<BlockInfoV1>> {
        let resolved = self.resolve_pool_blocks(slug, before_height, limit)?;
        self.pool_blocks_resolved(resolved)
    }
}
