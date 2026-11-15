use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_vecs::{
    LazyIndexedVec, LazyPerBlockCumulativeRolling, LazyWindowStartVec, shapes::Windows,
};
use brk_types::{Height, Version};
use vecdb::ReadableCloneableVec;

/// Outputs other than `OP_RETURN` ones: every output count minus theirs. The other outputs that
/// can never be spent (genesis, the BIP30-overwritten coinbase outputs, scripts over 10,000 bytes)
/// stay counted.
pub fn non_op_return_output_count(
    version: Version,
    op_return_count: &impl ReadableCloneableVec<Height, Count>,
    mappings: &MappingsVecs,
    window_starts: &Windows<&LazyWindowStartVec>,
) -> LazyPerBlockCumulativeRolling<Count> {
    let cumulative = LazyIndexedVec::new(
        "non_op_return_output_count_cumulative",
        version,
        op_return_count,
        &mappings.output_count_source(),
        |_, op_return, total| total - op_return,
    );
    LazyPerBlockCumulativeRolling::from_cumulative_source(
        "non_op_return_output_count",
        version,
        &cumulative,
        window_starts,
        mappings,
    )
}
