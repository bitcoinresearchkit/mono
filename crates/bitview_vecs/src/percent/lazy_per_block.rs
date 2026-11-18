use bitview_compute::{FixedRatio, NumericValue};
use bitview_primitives::Percent;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{BinaryTransform, Ident, ReadableCloneableVec, UnaryTransform};

use crate::{
    IndexSources, LazyIndexedVec, LazyPerBlock, ops::FixedToPercent, shapes::PercentViews,
};

/// A fixed-point view and the percentage view derived from it.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazyPercentPerBlock<B: FixedRatio>(
    pub PercentViews<LazyPerBlock<B, B>, LazyPerBlock<Percent, B>>,
);

impl<B: FixedRatio> LazyPercentPerBlock<B> {
    /// Inputs own their caches; the fixed-point and percent views retain no history.
    pub fn from_ratio<S, D, F>(
        name: &str,
        version: Version,
        numerator: &impl ReadableCloneableVec<Height, S>,
        denominator: &impl ReadableCloneableVec<Height, D>,
        indexes: &IndexSources,
    ) -> Self
    where
        S: NumericValue,
        D: NumericValue,
        F: BinaryTransform<S, D, B> + Send + Sync + 'static,
    {
        let source = LazyIndexedVec::new(
            &format!("{name}_{}_source", B::SUFFIX),
            version,
            numerator,
            denominator,
            |_, numerator, denominator| F::apply(numerator, denominator),
        );
        Self::from_height_source(name, version, &source, indexes)
    }

    pub fn from_height_source<V>(
        name: &str,
        version: Version,
        source: &V,
        indexes: &IndexSources,
    ) -> Self
    where
        V: ReadableCloneableVec<Height, B> + ?Sized,
    {
        let fixed_name = format!("{name}_{}", B::SUFFIX);
        let fixed =
            LazyPerBlock::from_height_source::<Ident>(&fixed_name, version, source, indexes);
        Self::from_fixed(name, version, fixed)
    }

    pub(crate) fn from_lazy_percent<F: UnaryTransform<B, B>>(
        name: &str,
        version: Version,
        source: &Self,
    ) -> Self {
        let fixed = LazyPerBlock::from_lazy::<F, B>(
            &format!("{name}_{}", B::SUFFIX),
            version,
            &source.fixed,
        );
        Self::from_fixed(name, version, fixed)
    }

    fn from_fixed(name: &str, version: Version, fixed: LazyPerBlock<B, B>) -> Self {
        let percent = LazyPerBlock::from_lazy::<FixedToPercent, B>(name, version, &fixed);
        Self(PercentViews { fixed, percent })
    }
}
