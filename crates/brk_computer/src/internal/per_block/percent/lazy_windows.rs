use std::marker::PhantomData;

use brk_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{
    BinaryTransform, CachedBoxedVec, DeltaAvg, LazyDeltaVec, ReadOnlyClone, ReadableCloneableVec,
    UnaryTransform,
};

use crate::{
    indexes,
    internal::{CachedWindowStartVec, FixedRatio, LazyRollingRatioVec, NumericValue, Windows},
};

use super::LazyPercentPerBlock;

struct ReverseOperands<F>(PhantomData<F>);

impl<S, D, T, F> BinaryTransform<S, D, T> for ReverseOperands<F>
where
    F: BinaryTransform<D, S, T>,
{
    #[inline]
    fn apply(source: S, cached: D) -> T {
        F::apply(cached, source)
    }
}

/// Fully lazy rolling percent windows — 4 windows (24h, 1w, 1m, 1y),
/// each with lazy PPM + lazy ratio/percent float views.
///
/// No stored vecs. All values are derived from one source.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyPercentRollingWindows<B: FixedRatio>(pub Windows<LazyPercentPerBlock<B>>);

impl<B: FixedRatio> LazyPercentRollingWindows<B> {
    pub(crate) fn from_cumulative_ratio<S, D, F>(
        name: &str,
        version: Version,
        numerator: &(impl ReadableCloneableVec<Height, S> + 'static),
        denominator: CachedBoxedVec<Height, D>,
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &indexes::Vecs,
    ) -> Self
    where
        S: NumericValue,
        D: NumericValue,
        F: BinaryTransform<S, D, B> + Send + Sync + 'static,
    {
        Self::from_cumulative_operands::<S, D, F>(
            name,
            version,
            numerator,
            denominator,
            cached_starts,
            indexes,
        )
    }

    pub(crate) fn from_cumulative_ratio_with_cached_numerator<S, D, F>(
        name: &str,
        version: Version,
        numerator: CachedBoxedVec<Height, S>,
        denominator: &(impl ReadableCloneableVec<Height, D> + 'static),
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &indexes::Vecs,
    ) -> Self
    where
        S: NumericValue,
        D: NumericValue,
        F: BinaryTransform<S, D, B> + Send + Sync + 'static,
    {
        Self::from_cumulative_operands::<D, S, ReverseOperands<F>>(
            name,
            version,
            denominator,
            numerator,
            cached_starts,
            indexes,
        )
    }

    fn from_cumulative_operands<S, D, F>(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + 'static),
        cached: CachedBoxedVec<Height, D>,
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &indexes::Vecs,
    ) -> Self
    where
        S: NumericValue,
        D: NumericValue,
        F: BinaryTransform<S, D, B> + Send + Sync + 'static,
    {
        let source = source.read_only_boxed_clone();

        Self(cached_starts.map_with_suffix(|suffix, cached_start| {
            let full_name = format!("{name}_{suffix}");
            let ratio = LazyRollingRatioVec::<S, D, B, F>::new(
                &format!("{full_name}_{}_source", B::SUFFIX),
                version,
                source.clone(),
                cached.clone(),
                cached_start.read_only_cached_boxed_clone(),
            );
            LazyPercentPerBlock::from_height_source(&full_name, version, ratio, indexes)
        }))
    }

    /// Rolling percentages derived from one cumulative in-memory source,
    /// without adding the four full-height averages to `cache_budget`.
    pub(crate) fn from_uncached_cumulative_average<T>(
        name: &str,
        version: Version,
        cumulative: &(impl ReadableCloneableVec<Height, T> + 'static),
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &indexes::Vecs,
    ) -> Self
    where
        T: NumericValue + JsonSchema,
    {
        let cumulative_source = cumulative.read_only_boxed_clone();

        Self(cached_starts.map_with_suffix(|suffix, cached_start| {
            let full_name = format!("{name}_{suffix}");
            let cached = cached_start.read_only_clone();
            let starts_version = cached.version();
            let average = LazyDeltaVec::<Height, T, B, DeltaAvg>::new(
                &format!("{full_name}_{}_source", B::SUFFIX),
                version,
                cumulative_source.clone(),
                starts_version,
                move || cached.snapshot(),
            );

            LazyPercentPerBlock::from_uncached_height_source(&full_name, version, average, indexes)
        }))
    }

    pub(crate) fn from_lazy_rolling<F: UnaryTransform<B, B>>(
        name: &str,
        version: Version,
        source: &Self,
    ) -> Self {
        Self(source.0.map_with_suffix(|suffix, source_window| {
            LazyPercentPerBlock::from_lazy_percent::<F>(
                &format!("{name}_{suffix}"),
                version,
                source_window,
            )
        }))
    }
}
