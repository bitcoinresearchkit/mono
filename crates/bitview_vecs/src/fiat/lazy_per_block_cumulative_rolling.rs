use bitview_collections::Windows;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::ReadableCloneableVec;

use crate::{
    FiatType, IndexSources, LazyFiatPerBlockCumulativeWithSums, LazyRollingAvgFiatFromHeight,
};

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazyFiatPerBlockCumulativeRolling<C: FiatType> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    inner: LazyFiatPerBlockCumulativeWithSums<C>,
    average: Windows<LazyRollingAvgFiatFromHeight<C>>,
}

impl<C: FiatType> LazyFiatPerBlockCumulativeRolling<C> {
    pub fn from_cumulative_cents_source(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, C> + ?Sized),
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Self {
        let inner = LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
            name,
            version,
            source,
            indexes,
            window_starts,
        );
        let average = window_starts.map_with_suffix(|suffix, window_start| {
            LazyRollingAvgFiatFromHeight::new(
                &format!("{name}_average_{suffix}"),
                version,
                &inner.cumulative.cents.height,
                *window_start,
                indexes,
            )
        });

        Self { inner, average }
    }
}
