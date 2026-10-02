use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPercentPerBlock, LazyWindowStartVec};
use brk_types::{Height, PartsPerMillion32, PoolSlug, StoredU64};
use vecdb::{LazyVec, ReadableCloneableVec, Version};

use super::{PoolHeights, pool_heights::BlocksMined};

fn pool_dominance(height: Height, blocks_mined: StoredU64) -> PartsPerMillion32 {
    PartsPerMillion32::from(u64::from(blocks_mined) as f64 / (u64::from(height) + 1) as f64)
}

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Block counts for a mining pool, using the per-height pool
    /// attribution series.
    pub blocks_mined: BlocksMined,
    /// Share of all blocks from genesis through the represented height
    /// attributed to a mining pool: cumulative pool block count divided by
    /// block height plus one.
    pub dominance: LazyPercentPerBlock<PartsPerMillion32>,
}

impl Vecs {
    pub fn forced_import(
        slug: PoolSlug,
        pool_heights: PoolHeights,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        let suffix = |s: &str| format!("{}_{s}", slug);

        let blocks_mined = BlocksMined::new(
            &suffix("blocks_mined"),
            slug,
            pool_heights,
            version + Version::ONE,
            mappings,
            window_starts,
        );

        let dominance_name = suffix("dominance");
        let dominance_source = LazyVec::init(
            &format!("{dominance_name}_ppm_source"),
            version,
            blocks_mined.cumulative.height.read_only_boxed_clone(),
            pool_dominance,
        );
        let dominance = LazyPercentPerBlock::from_height_source(
            &dominance_name,
            version,
            &dominance_source,
            mappings,
        );

        Self {
            blocks_mined,
            dominance,
        }
    }
}
