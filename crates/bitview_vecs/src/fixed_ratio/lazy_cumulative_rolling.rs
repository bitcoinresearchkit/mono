use bitview_collections::Windows;
use bitview_compute::{FixedRatio, NumericValue};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{BinaryTransform, ReadableCloneableVec, UnaryTransform};

use crate::{IndexSources, LazyFixedRatioPerBlock, LazyFixedRatioRollingWindows};

/// Lazy fixed-ratio views of a cumulative and its rolling windows — no stored vecs.
///
/// Cumulative and rolling window fields are both flattened to the same tree
/// level, so consumers see `{ ppm, percent, ratio, _24h, _1w, _1m, _1y }`.
#[derive(Clone, Traversable)]
pub struct LazyFixedRatioCumulativeRolling<B: FixedRatio> {
    #[traversable(flatten)]
    cumulative: LazyFixedRatioPerBlock<B>,
    #[traversable(flatten)]
    rolling: LazyFixedRatioRollingWindows<B>,
}

impl<B: FixedRatio> LazyFixedRatioCumulativeRolling<B> {
    /// Derive cumulative and rolling ratios from one potentially disk-backed
    /// cumulative numerator and aligned denominator/window metadata.
    /// Pass the readers shared with their owners; views request only the
    /// needed value ranges without changing cache retention.
    pub fn from_cumulative_ratio<S, D, F>(
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
        let cumulative = LazyFixedRatioPerBlock::from_ratio::<S, D, F>(
            name,
            version,
            numerator,
            denominator,
            indexes,
        );
        let rolling = LazyFixedRatioRollingWindows::from_cumulative_ratio::<S, D, F>(
            name,
            version,
            numerator,
            denominator,
            window_starts,
            indexes,
        );
        Self {
            cumulative,
            rolling,
        }
    }

    /// Same ratio, with the pinned value as numerator and the sole
    /// potentially disk-backed source as denominator.
    pub fn from_cumulative_ratio_with_numerator<S, D, F>(
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
        let cumulative = LazyFixedRatioPerBlock::from_ratio_with_numerator::<S, D, F>(
            name,
            version,
            numerator,
            denominator,
            indexes,
        );
        let rolling = LazyFixedRatioRollingWindows::from_cumulative_ratio_with_numerator::<S, D, F>(
            name,
            version,
            numerator,
            denominator,
            window_starts,
            indexes,
        );
        Self {
            cumulative,
            rolling,
        }
    }

    pub fn from_lazy_source<F: UnaryTransform<B, B>>(
        name: &str,
        version: Version,
        source: &Self,
    ) -> Self {
        let cumulative =
            LazyFixedRatioPerBlock::from_lazy_fixed_ratio::<F>(name, version, &source.cumulative);
        let rolling =
            LazyFixedRatioRollingWindows::from_lazy_rolling::<F>(name, version, &source.rolling);
        Self {
            cumulative,
            rolling,
        }
    }
}
