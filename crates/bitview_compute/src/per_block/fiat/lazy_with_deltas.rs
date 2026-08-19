use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::ReadableBoxedVec;

use crate::{
    CachedWindowStartVec, FiatType, FixedRatio, Identity, LazyFiatPerBlock, LazyPerBlock,
    LazyRollingDeltasFiatFromHeight, Windows,
};

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazyFiatPerBlockWithDeltas<C, CS, B>
where
    C: FiatType + Into<f64>,
    CS: FiatType + From<f64>,
    B: FixedRatio + From<f64>,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub inner: LazyFiatPerBlock<C>,
    pub delta: LazyRollingDeltasFiatFromHeight<C, CS, B>,
}

impl<C, CS, B> LazyFiatPerBlockWithDeltas<C, CS, B>
where
    C: FiatType + JsonSchema + Into<f64>,
    CS: FiatType + From<f64>,
    B: FixedRatio + From<f64>,
{
    pub fn from_boxed_cents_source(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Height, C>,
        delta_version_offset: Version,
        indexes: &crate::IndexSources,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let source = LazyPerBlock::from_boxed_height_source::<Identity<C>>(
            &format!("{name}_cents"),
            version,
            source,
            indexes,
        );
        let inner = LazyFiatPerBlock::from_lazy(name, version, &source);
        let delta = LazyRollingDeltasFiatFromHeight::new(
            &format!("{name}_delta"),
            version + delta_version_offset,
            &source.height,
            cached_starts,
            indexes,
        );
        Self { inner, delta }
    }
}
