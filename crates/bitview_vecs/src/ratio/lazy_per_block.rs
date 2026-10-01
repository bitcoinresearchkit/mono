use bitview_compute::{ComputedVecValue, FixedRatio, NumericValue};
use bitview_transforms::price_ratio;
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, PriceRatio, StoredF32, Version};
use schemars::JsonSchema;
use vecdb::{Ident, ReadableCloneableVec, UnaryTransform};

use crate::{IndexSources, LazyIndexedVec, LazyPerBlock};

const PRICE_RATIO_VERSION: Version = Version::new(5);

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

impl LazyRatioPerBlock<PriceRatio> {
    /// Reuse the standard spot/reference-price ratio policy for a price source.
    pub fn from_price_source(
        name: &str,
        version: Version,
        price: &impl ReadableCloneableVec<Height, Cents>,
        spot: &impl ReadableCloneableVec<Height, Cents>,
        indexes: &IndexSources,
    ) -> Self {
        let version = version + PRICE_RATIO_VERSION;
        let source = LazyIndexedVec::new(
            &format!("{name}_ppm_source"),
            version,
            price,
            spot,
            |_, price, spot| price_ratio(spot, price),
        );
        Self::from_height_source(name, version, &source, indexes)
    }
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
        source: &V,
        indexes: &IndexSources,
    ) -> Self
    where
        V: ReadableCloneableVec<Height, R> + ?Sized,
    {
        let ppm = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_{}", R::SUFFIX),
            version,
            source,
            indexes,
        );
        let ratio = LazyPerBlock::from_lazy::<R::ToRatio, R>(name, version, &ppm);

        Self { ppm, ratio }
    }
}
