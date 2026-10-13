use bitview_compute::{FixedRatio, NumericValue};
use bitview_primitives::Ratio;
use bitview_transforms::FixedToRatio;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{BinaryTransform, Ident, ReadableCloneableVec};

use crate::{IndexSources, LazyIndexedVec, LazyPerBlock};

/// Fully lazy variant of `RatioPerBlock` derived from one per-block source.
#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct LazyRatioPerBlock<R, S = R>
where
    R: FixedRatio,
    S: NumericValue + JsonSchema,
{
    /// Fixed-point storage: parts per million (1,000,000 represents 1.0) or basis points.
    #[traversable(hidden)]
    pub fixed: LazyPerBlock<R, S>,
    /// As a ratio.
    ratio: LazyPerBlock<Ratio, R>,
}

impl<R> LazyRatioPerBlock<R>
where
    R: FixedRatio,
{
    /// `F` of two per-block sources; they own their caches, the views retain no history.
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
        F: BinaryTransform<S, D, R> + Send + Sync + 'static,
    {
        let source = LazyIndexedVec::new(
            &format!("{name}_{}_source", R::SUFFIX),
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
        V: ReadableCloneableVec<Height, R> + ?Sized,
    {
        let fixed = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_{}", R::SUFFIX),
            version,
            source,
            indexes,
        );
        let ratio = LazyPerBlock::from_lazy::<FixedToRatio, R>(name, version, &fixed);

        Self { fixed, ratio }
    }
}
