use bitview_compute::{FixedRatio, NumericValue};
use bitview_primitives::Ratio;
use bitview_transforms::FixedToRatio;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{Ident, ReadableCloneableVec, UnaryTransform};

use crate::{IndexSources, LazyPerBlock, Resolutions};

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

impl<R, S> LazyRatioPerBlock<R, S>
where
    R: FixedRatio,
    S: NumericValue + JsonSchema,
{
    /// Transform one stored per-block source.
    pub fn from_resolutions<F>(name: &str, version: Version, source: &Resolutions<S>) -> Self
    where
        F: UnaryTransform<S, R>,
    {
        let fixed =
            LazyPerBlock::from_resolutions::<F>(&format!("{name}_{}", R::SUFFIX), version, source);
        let ratio = LazyPerBlock::from_lazy::<FixedToRatio, S>(name, version, &fixed);

        Self { fixed, ratio }
    }
}

impl<R> LazyRatioPerBlock<R>
where
    R: FixedRatio,
{
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
