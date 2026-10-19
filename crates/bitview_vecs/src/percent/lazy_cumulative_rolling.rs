use bitview_compute::{FixedRatio, NumericValue};
use bitview_primitives::{Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{BinaryTransform, ReadableCloneableVec, UnaryTransform};

use crate::{
    IndexSources, LazyPerBlockCumulativeRolling, LazyPercentPerBlock, LazyPercentRollingWindows,
    ops::Quotient, shapes::Windows,
};

/// A count from its running total, `{name}_count`, and its share of `total`'s running total
/// over the same windows, `{name}_share`.
pub fn count_and_share(
    name: &str,
    version: Version,
    cumulative: &(impl ReadableCloneableVec<Height, Count> + ?Sized),
    total: &impl ReadableCloneableVec<Height, Count>,
    window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    indexes: &IndexSources,
) -> (
    LazyPerBlockCumulativeRolling<Count>,
    LazyPercentCumulativeRolling<PartsPerMillion32>,
) {
    let count = LazyPerBlockCumulativeRolling::from_cumulative_source(
        &format!("{name}_count"),
        version,
        cumulative,
        window_starts,
        indexes,
    );
    let share = LazyPercentCumulativeRolling::from_cumulative_ratio::<
        Count,
        Count,
        Quotient<PartsPerMillion32>,
    >(
        &format!("{name}_share"),
        version,
        &count.cumulative.height,
        total,
        window_starts,
        indexes,
    );
    (count, share)
}

/// Cumulative and rolling views are flattened into `{ cumulative, 24h, 1w, 1m, 1y }`, with ids
/// `{name}_cumulative` and `{name}_{window}`.
#[derive(Clone, Traversable)]
pub struct LazyPercentCumulativeRolling<B: FixedRatio> {
    /// Numerator and denominator accumulated from the first block through the
    /// represented block.
    #[traversable(flatten)]
    cumulative: LazyPercentPerBlock<B>,
    #[traversable(flatten)]
    rolling: LazyPercentRollingWindows<B>,
}

impl<B: FixedRatio> LazyPercentCumulativeRolling<B> {
    /// Derive cumulative and rolling ratios from cumulative numerator and denominator readers.
    /// Pass the readers shared with their owners; views request only the needed value ranges
    /// without changing cache retention.
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
        let cumulative = LazyPercentPerBlock::from_ratio::<S, D, F>(
            &format!("{name}_cumulative"),
            version,
            numerator,
            denominator,
            indexes,
        );
        let rolling = LazyPercentRollingWindows::from_cumulative_ratio::<S, D, F>(
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
        let cumulative = LazyPercentPerBlock::from_lazy_percent::<F>(
            &format!("{name}_cumulative"),
            version,
            &source.cumulative,
        );
        let rolling =
            LazyPercentRollingWindows::from_lazy_rolling::<F>(name, version, &source.rolling);
        Self {
            cumulative,
            rolling,
        }
    }
}
