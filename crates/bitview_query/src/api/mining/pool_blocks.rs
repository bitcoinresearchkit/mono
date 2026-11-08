use bitcoin::hashes::{Hash, HashEngine, sha256};
use bitview_primitives::PoolSlug;
use bitview_types::BlockInfoV1;
use brk_types::{BlockHash, Dollars, Height};
use vecdb::ReadableVec;

use crate::{Error, OptionData, Query, ResolvedBlocks, Result, api::block::blocks_v1_identity};

/// A pool-block page resolved against one exact published chain view.
pub struct ResolvedPoolBlocks {
    chain: ResolvedBlocks,
    heights: Vec<Height>,
    /// In the same descending order as the heights.
    prices: Vec<Dollars>,
    activity_anchor: Option<BlockHash>,
}

impl ResolvedPoolBlocks {
    /// Identifies the page: the extended rows' identity at the pool's last activity, and the
    /// selected heights, as a reclassification can change older rows at the same anchor.
    pub fn identity(&self) -> sha256::Hash {
        let mut engine = sha256::Hash::engine();
        engine.input(blocks_v1_identity(self.activity_anchor, &self.prices).as_byte_array());
        for height in &self.heights {
            engine.input(&u32::from(*height).to_le_bytes());
        }
        sha256::Hash::from_engine(engine)
    }
}

impl Query {
    /// Resolve the page's exact block heights and activity anchor once: the pool's latest
    /// `limit` blocks strictly below `before_height` (default through the tip), so a page
    /// requested before its predecessor's last height continues it, as mempool.space pages.
    pub fn resolve_pool_blocks(
        &self,
        slug: PoolSlug,
        before_height: Option<Height>,
        limit: usize,
    ) -> Result<ResolvedPoolBlocks> {
        let chain = self.blocks_window(None, 0)?;
        let tip = chain.last_height().ok_or(Error::StateUpdating)?;
        let heights = before_height
            .map_or(Some(tip), Height::decremented)
            .map_or_else(Vec::new, |through| {
                self.plugins()
                    .pools
                    .heights
                    .latest_heights(slug, through.min(tip), limit)
            });
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
            chain,
            heights,
            prices,
            activity_anchor,
        })
    }

    /// Load a resolved page without repeating its pool-height lookup.
    pub fn pool_blocks_resolved(&self, resolved: ResolvedPoolBlocks) -> Result<Vec<BlockInfoV1>> {
        let ResolvedPoolBlocks {
            chain,
            heights,
            prices,
            ..
        } = resolved;
        chain.build_v1_heights(self, &heights, &prices)
    }
}
