use brk_traversable::Traversable;
use brk_types::{Height, PartsPerMillion32, PoolSlug, StoredU64};
use vecdb::{LazyVec, ReadableCloneableVec, Version};

mod blocks_mined;

use blocks_mined::BlocksMined;

use crate::{
    indexes,
    internal::{CachedWindowStartVec, LazyPercentPerBlock, Windows},
};

use super::PoolHeights;

fn pool_dominance(height: Height, blocks_mined: StoredU64) -> PartsPerMillion32 {
    PartsPerMillion32::from(u64::from(blocks_mined) as f64 / (u64::from(height) + 1) as f64)
}

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Block counts for the selected mining pool, using the per-height pool
    /// attribution series.
    pub blocks_mined: BlocksMined,
    /// Share of all blocks from genesis through the represented height
    /// attributed to the selected pool: cumulative pool block count divided by
    /// block height plus one.
    pub dominance: LazyPercentPerBlock<PartsPerMillion32>,
}

impl Vecs {
    pub(crate) fn forced_import(
        slug: PoolSlug,
        pool_heights: PoolHeights,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let suffix = |s: &str| format!("{}_{s}", slug);

        let blocks_mined = BlocksMined::new(
            &suffix("blocks_mined"),
            slug,
            pool_heights,
            version + Version::ONE,
            indexes,
            cached_starts,
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
            dominance_source,
            indexes,
        );

        Self {
            blocks_mined,
            dominance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dominance_is_cumulative_share_of_chain() {
        assert_eq!(
            pool_dominance(Height::from(3_u32), StoredU64::from(1_u64)),
            PartsPerMillion32::from(0.25),
        );
        assert_eq!(
            pool_dominance(Height::from(9_u32), StoredU64::from(10_u64)),
            PartsPerMillion32::ONE,
        );
    }
}
