use bitview_collections::Windows;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::StoredU64;
use bitview_transforms::BlockCountTarget;
use bitview_vecs::{ConstantVecs, LazyPerBlockCumulativeRolling, LazyWindowStartVec};
use brk_types::{Height, Version};
use vecdb::{IndexVec, ReadOnlyClone};

use super::Vecs;

fn cumulative_block_count(height: Height) -> StoredU64 {
    StoredU64::from(u64::from(height) + 1)
}

impl Vecs {
    pub(crate) fn new(
        version: Version,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        let total_source = IndexVec::new(
            "block_count_cumulative_source",
            version + Version::ONE,
            indexer.vecs().blocks.weight.read_only_clone(),
            cumulative_block_count,
        );

        Self {
            target: Windows {
                _24h: ConstantVecs::new::<BlockCountTarget<1>>(
                    "block_count_target_24h",
                    version,
                    mappings,
                ),
                _1w: ConstantVecs::new::<BlockCountTarget<7>>(
                    "block_count_target_1w",
                    version,
                    mappings,
                ),
                _1m: ConstantVecs::new::<BlockCountTarget<30>>(
                    "block_count_target_1m",
                    version,
                    mappings,
                ),
                _1y: ConstantVecs::new::<BlockCountTarget<365>>(
                    "block_count_target_1y",
                    version,
                    mappings,
                ),
            },
            total: LazyPerBlockCumulativeRolling::from_cumulative_source(
                "block_count",
                version + Version::ONE,
                &total_source,
                window_starts,
                mappings,
            ),
        }
    }
}
