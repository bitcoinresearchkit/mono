use bitview_collections::Windows;
use bitview_compute::{FixedRatio, NumericValue};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{
    BinaryTransform, DeltaAvg, LazyDeltaVec, ReadableCloneableVec, ReverseOperands, UnaryTransform,
};

use crate::{IndexSources, LazyPercentPerBlock, LazyRollingRatioVec};

/// Fully lazy rolling percent windows — 4 windows (24h, 1w, 1m, 1y),
/// each with lazy fixed-point values and a lazy percent view.
///
/// No stored vecs. All values are derived from one source.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyPercentRollingWindows<B: FixedRatio>(pub Windows<LazyPercentPerBlock<B>>);

impl<B: FixedRatio> LazyPercentRollingWindows<B> {
    pub(crate) fn from_cumulative_ratio<S, D, F>(
        name: &str,
        version: Version,
        numerator: &impl ReadableCloneableVec<Height, S>,
        denominator: &impl ReadableCloneableVec<Height, D>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
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
            window_starts,
            indexes,
        )
    }

    pub(crate) fn from_cumulative_ratio_with_numerator<S, D, F>(
        name: &str,
        version: Version,
        numerator: &impl ReadableCloneableVec<Height, S>,
        denominator: &impl ReadableCloneableVec<Height, D>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
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
            window_starts,
            indexes,
        )
    }

    fn from_cumulative_operands<S, D, F>(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        operand: &impl ReadableCloneableVec<Height, D>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self
    where
        S: NumericValue,
        D: NumericValue,
        F: BinaryTransform<S, D, B> + Send + Sync + 'static,
    {
        Self(window_starts.map_with_suffix(|suffix, window_start| {
            let full_name = format!("{name}_{suffix}");
            let ratio = LazyRollingRatioVec::<S, D, B, F>::new(
                &format!("{full_name}_{}_source", B::SUFFIX),
                version,
                source,
                operand,
                *window_start,
            );
            LazyPercentPerBlock::from_height_source(&full_name, version, &ratio, indexes)
        }))
    }

    /// Derive each trailing average from one cumulative reader and window starts.
    pub fn from_cumulative_average<T>(
        name: &str,
        version: Version,
        cumulative: &impl ReadableCloneableVec<Height, T>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self
    where
        T: NumericValue + JsonSchema,
    {
        let cumulative_source = cumulative.read_only_boxed_clone();

        Self(window_starts.map_with_suffix(|suffix, window_start| {
            let full_name = format!("{name}_{suffix}");
            let operand = window_start.read_only_boxed_clone();
            let average = LazyDeltaVec::<Height, T, B, DeltaAvg>::new(
                &format!("{full_name}_{}_source", B::SUFFIX),
                version,
                cumulative_source.clone(),
                operand,
            );

            LazyPercentPerBlock::from_height_source(&full_name, version, &average, indexes)
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
