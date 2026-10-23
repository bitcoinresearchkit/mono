use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_vecs::{LazyCumulativeRolling, LazyWindowStartVec, shapes::Windows};
use brk_types::{Height, Version};
use vecdb::{IndexVec, ReadOnlyClone};

fn cumulative_block_count(height: Height) -> Count {
    Count::from(u64::from(height) + 1)
}

pub(crate) fn block_count(
    version: Version,
    indexer: &Indexer,
    mappings: &MappingsVecs,
    window_starts: &Windows<&LazyWindowStartVec>,
) -> LazyCumulativeRolling<Count> {
    let total_source = IndexVec::new(
        "block_count_cumulative_source",
        version + Version::ONE,
        (*indexer.vecs().blocks.weight).read_only_clone(),
        cumulative_block_count,
    );
    LazyCumulativeRolling::from_cumulative_source(
        "block_count",
        version + Version::ONE,
        &total_source,
        window_starts,
        mappings,
    )
}
