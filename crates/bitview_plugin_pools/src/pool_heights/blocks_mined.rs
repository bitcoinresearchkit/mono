use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Count, PoolSlug};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyPerBlock, LazyPreviousDeltaVec, LazyRollingSumsFromHeight, LazyWindowStartVec,
};
use brk_types::Height;
use vecdb::{Ident, Version};

use super::{PoolCumulativeVec, PoolHeights};

#[derive(Clone, Traversable)]
pub struct BlocksMined {
    /// One when the represented block is attributed to a mining pool;
    /// otherwise zero.
    pub block: LazyPreviousDeltaVec<Height, Count>,
    /// Number of blocks attributed to a mining pool from genesis through
    /// the represented height, inclusive.
    pub cumulative: LazyPerBlock<Count>,
    pub sum: LazyRollingSumsFromHeight<Count>,
}

impl BlocksMined {
    pub fn new(
        name: &str,
        slug: PoolSlug,
        pool_heights: PoolHeights,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        let cumulative_name = format!("{name}_cumulative");
        let cumulative_source = PoolCumulativeVec::new(&cumulative_name, slug, pool_heights);
        let cumulative = LazyPerBlock::from_height_source::<Ident>(
            &cumulative_name,
            version,
            &cumulative_source,
            mappings,
        );
        let block = LazyPreviousDeltaVec::new(name, version, &cumulative.height);
        let sum = LazyRollingSumsFromHeight::new(
            &format!("{name}_sum"),
            version,
            &cumulative.height,
            window_starts,
            mappings,
        );

        Self {
            block,
            cumulative,
            sum,
        }
    }
}
