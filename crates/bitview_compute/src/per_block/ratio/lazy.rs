use bitview_traversable::Traversable;
use brk_types::{Height, StoredF32, Version};
use schemars::JsonSchema;
use vecdb::{ReadableVec, TypedVec, UnaryTransform};

use crate::{ComputedVecValue, FixedRatio, Identity, LazyPerBlock, NumericValue};

/// Fully lazy variant of `RatioPerBlock` derived from one per-block source.
#[derive(Clone, Traversable)]
pub struct LazyRatioPerBlock<R, S = R>
where
    R: FixedRatio,
    S: NumericValue + JsonSchema,
{
    /// Unitless ratio in parts per million; 1,000,000 represents 1.0.
    pub ppm: LazyPerBlock<R, S>,
    /// Unitless decimal ratio derived as parts per million divided by 1,000,000.
    pub ratio: LazyPerBlock<StoredF32, R>,
}

impl<R, S> LazyRatioPerBlock<R, S>
where
    R: FixedRatio,
    S: NumericValue + JsonSchema,
{
    pub fn from_lazy_source<F, S2T>(
        name: &str,
        version: Version,
        source: &LazyPerBlock<S, S2T>,
    ) -> Self
    where
        F: UnaryTransform<S, R>,
        S2T: ComputedVecValue + JsonSchema,
    {
        let ppm =
            LazyPerBlock::from_lazy::<F, S2T>(&format!("{name}_{}", R::SUFFIX), version, source);
        let ratio = LazyPerBlock::from_lazy::<R::ToRatio, S>(name, version, &ppm);

        Self { ppm, ratio }
    }
}

impl<R> LazyRatioPerBlock<R>
where
    R: FixedRatio,
{
    pub fn from_height_source<V>(
        name: &str,
        version: Version,
        source: V,
        indexes: &crate::IndexSources,
    ) -> Self
    where
        V: TypedVec<I = Height, T = R> + ReadableVec<Height, R> + Clone + 'static,
    {
        let ppm = LazyPerBlock::from_height_source::<Identity<R>>(
            &format!("{name}_{}", R::SUFFIX),
            version,
            source,
            indexes,
        );
        let ratio = LazyPerBlock::from_lazy::<R::ToRatio, R>(name, version, &ppm);

        Self { ppm, ratio }
    }
}
