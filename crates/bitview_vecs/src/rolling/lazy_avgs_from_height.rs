use bitview_collections::Windows;
use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::ReadableCloneableVec;

use crate::{IndexSources, LazyRollingAvgFromHeight};

/// Lazy rolling averages for all 4 window durations (24h, 1w, 1m, 1y),
/// derived from a cumulative vec + cached window starts.
///
/// Nothing is stored on disk — all values are computed on-the-fly via
/// `LazyDeltaVec<Height, T, T::Fract, DeltaAvg>`: `(cum[h] - cum[start-1]) / (h - start + 1)`.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyRollingAvgsFromHeight<T>(
    /// Arithmetic mean of the per-block values over the trailing window ending
    /// at the represented block; each block has equal weight. At time-period
    /// indexes, the value is taken at the period's final block.
    pub Windows<LazyRollingAvgFromHeight<T>>,
)
where
    T: NumericValue + JsonSchema + Quantity;

impl<T> LazyRollingAvgsFromHeight<T>
where
    T: NumericValue + JsonSchema + Quantity,
{
    pub(crate) fn new(
        name: &str,
        version: Version,
        cumulative: &impl ReadableCloneableVec<Height, T>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        Self(window_starts.map_with_suffix(|suffix, window_start| {
            LazyRollingAvgFromHeight::from_source(
                &format!("{name}_{suffix}"),
                version,
                cumulative,
                *window_start,
                indexes,
            )
        }))
    }
}
