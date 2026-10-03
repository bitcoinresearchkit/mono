use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::StoredU64;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyIndexedVec, LazyPerBlockCumulativeRolling, LazyWindowStartVec};
use brk_types::{Height, Version};
use derive_more::Deref;
use vecdb::{ReadableCloneableVec, ReadableVec};

#[derive(Clone, Deref, Traversable)]
pub struct SpendableOutputCount {
    #[deref]
    #[traversable(flatten)]
    pub views: LazyPerBlockCumulativeRolling<StoredU64>,
}

impl SpendableOutputCount {
    pub fn new(
        version: Version,
        op_return_count: &impl ReadableCloneableVec<Height, StoredU64>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        let cumulative = LazyIndexedVec::new(
            "spendable_output_count_cumulative",
            version,
            op_return_count,
            &mappings.output_count(),
            |_, op_return, total| total - op_return,
        );
        let views = LazyPerBlockCumulativeRolling::from_cumulative_source(
            "spendable_output_count",
            version,
            &cumulative,
            window_starts,
            mappings,
        );

        Self { views }
    }

    pub fn cumulative_source(
        &self,
    ) -> &(impl ReadableVec<Height, StoredU64> + Clone + 'static + use<>) {
        &self.views.cumulative.height
    }
}
