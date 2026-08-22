use bitview_traversable::Traversable;
use derive_more::{Deref, DerefMut};
use vecdb::ReadableBoxedVec;

use brk_types::{Height, Version};

use crate::{
    CachedWindowStartVec, FiatType, LazyFiatPerBlockCumulativeWithSums,
    LazyRollingAvgFiatFromHeight, Windows,
};

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazyFiatPerBlockCumulativeRolling<C: FiatType> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub inner: LazyFiatPerBlockCumulativeWithSums<C>,
    pub average: Windows<LazyRollingAvgFiatFromHeight<C>>,
}

impl<C: FiatType> LazyFiatPerBlockCumulativeRolling<C> {
    pub fn from_boxed_cumulative_cents_source(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Height, C>,
        indexes: &crate::IndexSources,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let inner = LazyFiatPerBlockCumulativeWithSums::from_boxed_cumulative_cents_source(
            name,
            version,
            source,
            indexes,
            cached_starts,
        );
        let average = cached_starts.map_with_suffix(|suffix, cached_start| {
            LazyRollingAvgFiatFromHeight::new(
                &format!("{name}_average_{suffix}"),
                version,
                &inner.cumulative.cents.height,
                cached_start,
                indexes,
            )
        });

        Self { inner, average }
    }
}
