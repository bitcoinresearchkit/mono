use brk_traversable::Traversable;
use brk_types::{Height, StoredU64, Version};
use derive_more::Deref;
use vecdb::{CachedBoxedVec, CachedReadableVec, CachedVec, ReadableCloneableVec};

use crate::{
    indexes,
    internal::{CachedWindowStartVec, LazyIndexedVec, LazyPerBlockCumulativeRolling, Windows},
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
    pub(crate) fn new(
        version: Version,
        op_return_count: &(impl ReadableCloneableVec<Height, StoredU64> + 'static),
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let cumulative = CachedVec::wrap(LazyIndexedVec::new(
            "spendable_output_count_cumulative",
            version,
            op_return_count.read_only_boxed_clone(),
            indexes.output_count(),
            |_, op_return, total| total - op_return,
        ));
        let views = LazyPerBlockCumulativeRolling::from_cached_cumulative_source(
            "spendable_output_count",
            version,
            cumulative.clone(),
            cached_starts,
            indexes,
        );

        Self { views, cumulative }
    }

    pub(crate) fn cached_cumulative(&self) -> CachedBoxedVec<Height, StoredU64> {
        self.cumulative.cached_boxed_clone()
    }

    pub(crate) fn invalidate(&self) {
        self.cumulative.invalidate();
    }
}
