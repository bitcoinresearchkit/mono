use brk_traversable::Traversable;
use brk_types::{Height, PoolSlug, StoredU64};
use vecdb::{ReadableCloneableVec, Version};

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, Identity, LazyPerBlock, LazyPreviousDeltaVec,
        LazyRollingSumsFromHeight, Windows,
    },
};

use super::super::{PoolHeights, pool_heights::PoolCumulativeVec};

#[derive(Clone, Traversable)]
pub struct BlocksMined {
    /// One when the represented block is attributed to the selected pool;
    /// otherwise zero.
    pub block: LazyPreviousDeltaVec<Height, StoredU64>,
    /// Number of blocks attributed to the selected pool from genesis through
    /// the represented height, inclusive.
    pub cumulative: LazyPerBlock<StoredU64>,
    pub sum: LazyRollingSumsFromHeight<StoredU64>,
}

impl BlocksMined {
    pub fn new(
        name: &str,
        slug: PoolSlug,
        pool_heights: PoolHeights,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let cumulative_name = format!("{name}_cumulative");
        let cumulative_source = PoolCumulativeVec::new(&cumulative_name, slug, pool_heights);
        let cumulative = LazyPerBlock::from_height_source::<Identity<StoredU64>>(
            &cumulative_name,
            version,
            cumulative_source,
            indexes,
        );
        let block =
            LazyPreviousDeltaVec::new(name, version, cumulative.height.read_only_boxed_clone());
        let sum = LazyRollingSumsFromHeight::from_compact_cumulative(
            &format!("{name}_sum"),
            version,
            &cumulative.height,
            cached_starts,
            indexes,
        );

        Self {
            block,
            cumulative,
            sum,
        }
    }
}
