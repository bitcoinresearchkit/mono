use brk_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::ReadableBoxedVec;

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, FiatType, FixedRatio, LazyFiatPerBlockCumulativeWithSums,
        LazyRollingDeltasFiatFromHeight, Windows,
    },
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
    pub inner: LazyFiatPerBlockCumulativeWithSums<C>,
    pub delta: LazyRollingDeltasFiatFromHeight<C, CS, B>,
}

impl<C, CS, B> LazyFiatPerBlockCumulativeWithSumsAndDeltas<C, CS, B>
where
    C: FiatType + Into<f64>,
    CS: FiatType + From<f64>,
    B: FixedRatio + From<f64>,
{
    pub(crate) fn from_boxed_cumulative_cents_source(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Height, C>,
        delta_version_offset: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let inner = LazyFiatPerBlockCumulativeWithSums::from_boxed_cumulative_cents_source(
            name,
            version,
            source,
            indexes,
            cached_starts,
        );
        let delta = LazyRollingDeltasFiatFromHeight::new(
            &format!("{name}_delta"),
            version + delta_version_offset,
            &inner.cumulative.cents.height,
            cached_starts,
            indexes,
        );
        Self { inner, delta }
    }
}
