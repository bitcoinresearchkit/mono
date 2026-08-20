use bitview_compute::{
    BlockCountTarget1m, BlockCountTarget1w, BlockCountTarget1y, BlockCountTarget24h, CACHE_BUDGET,
    CachedWindowStartVec, ConstantVecs, LazyPerBlockCumulativeRolling, Windows,
};
use bitview_plugin_indexer::Indexer;
use brk_types::{Height, StoredU64, Version, Weight};
use vecdb::{LazyVec, ReadableCloneableVec};

use super::Vecs;

fn cumulative_block_count(height: Height, _: Weight) -> StoredU64 {
    StoredU64::from(u64::from(height) + 1)
}

pub trait Import {
    fn new(
        version: Version,
        indexer: &Indexer,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self;
}

impl Import for Vecs {
    fn new(
        version: Version,
        indexer: &Indexer,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let total_source = LazyVec::init(
            "block_count_cumulative_source",
            version + Version::ONE,
            indexer.vecs().blocks.weight.read_only_boxed_clone(),
            cumulative_block_count,
        );
        let total_source = CACHE_BUDGET.wrap(total_source);

        Self {
            target: Windows {
                _24h: ConstantVecs::new::<BlockCountTarget24h>(
                    "block_count_target_24h",
                    version,
                    mappings,
                ),
                _1w: ConstantVecs::new::<BlockCountTarget1w>(
                    "block_count_target_1w",
                    version,
                    mappings,
                ),
                _1m: ConstantVecs::new::<BlockCountTarget1m>(
                    "block_count_target_1m",
                    version,
                    mappings,
                ),
                _1y: ConstantVecs::new::<BlockCountTarget1y>(
                    "block_count_target_1y",
                    version,
                    mappings,
                ),
            },
            total: LazyPerBlockCumulativeRolling::from_cumulative_source(
                "block_count",
                version + Version::ONE,
                total_source,
                cached_starts,
                mappings,
            ),
        }
    }
}
