use bitview_traversable::Traversable;
use brk_types::{Height, StoredU64, Version};
use derive_more::Deref;
use vecdb::{CachedBoxedVec, CachedReadableVec, CachedVec, ReadableCloneableVec};

use bitview_compute::{
    CachedWindowStartVec, LazyIndexedVec, LazyPerBlockCumulativeRolling, Windows,
};

#[derive(Clone, Deref, Traversable)]
pub struct CachedSpendableOutputCount {
    #[deref]
    #[traversable(flatten)]
    pub views: LazyPerBlockCumulativeRolling<StoredU64>,
    #[traversable(skip)]
    cumulative: CachedVec<LazyIndexedVec<Height, StoredU64, StoredU64, StoredU64>>,
}

impl CachedSpendableOutputCount {
    pub fn new(
        version: Version,
        op_return_count: &(impl ReadableCloneableVec<Height, StoredU64> + 'static),
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let cumulative = CachedVec::wrap(LazyIndexedVec::new(
            "spendable_output_count_cumulative",
            version,
            op_return_count.read_only_boxed_clone(),
            mappings.output_count(),
            |_, op_return, total| total - op_return,
        ));
        let views = LazyPerBlockCumulativeRolling::from_cumulative_source(
            "spendable_output_count",
            version,
            cumulative.clone(),
            cached_starts,
            mappings,
        );

        Self { views, cumulative }
    }

    pub fn cached_cumulative(&self) -> CachedBoxedVec<Height, StoredU64> {
        self.cumulative.cached_boxed_clone()
    }

    pub fn invalidate(&self) {
        self.cumulative.invalidate();
    }
}
