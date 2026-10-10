use bitview_collections::Windows;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_vecs::{LazyPerBlockCumulativeRolling, LazyWindowStartVec};
use brk_types::{Height, Version};
use vecdb::{IndexVec, ReadOnlyClone};

fn cumulative_block_count(height: Height) -> Count {
    Count::from(u64::from(height) + 1)
}

/// Block count: one per block, height plus one cumulatively, and the blocks of each trailing
/// window.
pub(crate) fn block_count(
    version: Version,
    indexer: &Indexer,
    mappings: &MappingsVecs,
    window_starts: &Windows<&LazyWindowStartVec>,
) -> LazyPerBlockCumulativeRolling<Count> {
    let total_source = IndexVec::new(
        "block_count_cumulative_source",
        version + Version::ONE,
        indexer.vecs().blocks.weight.read_only_clone(),
        cumulative_block_count,
    );
    LazyPerBlockCumulativeRolling::from_cumulative_source(
        "block_count",
        version + Version::ONE,
        &total_source,
        window_starts,
        mappings,
    )
}
