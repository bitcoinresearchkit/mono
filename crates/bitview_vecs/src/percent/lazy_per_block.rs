use bitview_collections::PercentViews;
use bitview_compute::{FixedRatio, NumericValue};
use bitview_primitives::Percent;
use bitview_transforms::FixedToPercent;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{BinaryTransform, Ident, ReadableCloneableVec, UnaryTransform, VecValue};

use crate::{IndexSources, LazyIndexedVec, LazyLookbackVec, LazyPerBlock};

/// Fully lazy variant of `PercentPerBlock` — no stored vecs.
///
/// Fixed-point values are lazily derived from one source, and the percent view is chained
/// from them.
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

    pub(crate) fn from_ratio_with_numerator<S, D, F>(
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
            denominator,
            numerator,
            |_, denominator, numerator| F::apply(numerator, denominator),
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

    /// Create from two values a fixed distance apart in one height source.
    pub fn from_lookback_source<S>(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        lookback: usize,
        compute: fn(S, Option<S>) -> B,
        indexes: &IndexSources,
    ) -> Self
    where
        S: VecValue,
    {
        let fixed_name = format!("{name}_{}", B::SUFFIX);
        let source = LazyLookbackVec::new(
            &format!("{fixed_name}_source"),
            version,
            source,
            lookback,
            compute,
        );
        let fixed =
            LazyPerBlock::from_height_source::<Ident>(&fixed_name, version, &source, indexes);

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
