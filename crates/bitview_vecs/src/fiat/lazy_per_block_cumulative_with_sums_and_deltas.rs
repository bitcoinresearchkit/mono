use bitview_collections::Windows;
use bitview_compute::FixedRatio;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::ReadableCloneableVec;

use crate::{
    FiatType, IndexSources, LazyFiatPerBlockCumulativeWithSums, LazyRollingDeltasFiatFromHeight,
};

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazyFiatPerBlockCumulativeWithSumsAndDeltas<C, CS, B>
where
    C: FiatType + Into<f64>,
    CS: FiatType + From<f64>,
    B: FixedRatio + From<f64>,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    inner: LazyFiatPerBlockCumulativeWithSums<C>,
    pub delta: LazyRollingDeltasFiatFromHeight<C, CS, B>,
}

impl<C, CS, B> LazyFiatPerBlockCumulativeWithSumsAndDeltas<C, CS, B>
where
    C: FiatType + Into<f64>,
    CS: FiatType + From<f64>,
    B: FixedRatio + From<f64>,
{
    pub fn from_cumulative_cents_source(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, C> + ?Sized),
        delta_version_offset: Version,
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
        let delta = LazyRollingDeltasFiatFromHeight::new(
            &format!("{name}_delta"),
            version + delta_version_offset,
            &inner.cumulative.cents.height,
            window_starts,
            indexes,
        );
        Self { inner, delta }
    }
}
