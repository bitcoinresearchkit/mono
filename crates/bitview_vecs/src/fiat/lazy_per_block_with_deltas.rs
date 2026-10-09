use bitview_collections::Windows;
use bitview_compute::FixedRatio;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Ident, ReadableCloneableVec};

use crate::{
    FiatType, IndexSources, LazyFiatPerBlock, LazyPerBlock, LazyRollingDeltasFiatFromHeight,
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
    #[traversable(flatten, rename = "block")]
    inner: LazyFiatPerBlock<C>,
    delta: LazyRollingDeltasFiatFromHeight<C, CS, B>,
}

impl<C, CS, B> LazyFiatPerBlockWithDeltas<C, CS, B>
where
    C: FiatType + JsonSchema + Into<f64>,
    CS: FiatType + From<f64>,
    B: FixedRatio + From<f64>,
{
    pub fn from_cents_source(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, C> + ?Sized),
        delta_version_offset: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Self {
        let source = LazyPerBlock::from_height_source::<Ident>(
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
            window_starts,
            indexes,
        );
        Self { inner, delta }
    }
}
