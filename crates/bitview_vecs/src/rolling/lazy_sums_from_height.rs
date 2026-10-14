use bitview_collections::Windows;
use bitview_compute::NumericValue;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::ReadableCloneableVec;

use crate::{IndexSources, LazyRollingSumFromHeight};

/// Lazy rolling sums for all 4 window durations (24h, 1w, 1m, 1y),
/// derived from a cumulative vec + cached window starts.
///
/// Nothing is stored on disk — all values are computed on-the-fly via
/// `LazyDeltaVec<Height, T, T, DeltaSub>`: `cum[h] - cum[window_start[h] - 1]`,
/// using zero when the window starts at genesis.
///
/// Implements `Traversable` to expose `_24h`, `_1w`, `_1m`, `_1y` with
/// the same tree structure as the old `RollingWindows<T>`.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazyRollingSumsFromHeight<T>(
    /// Total of the per-block values over the trailing window ending at the
    /// represented block. At time-period indexes, the value is taken at the
    /// period's final block.
    pub Windows<LazyRollingSumFromHeight<T>>,
)
where
    T: NumericValue + JsonSchema;

impl<T> LazyRollingSumsFromHeight<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn new(
        name: &str,
        version: Version,
        cumulative: &impl ReadableCloneableVec<Height, T>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        Self(window_starts.map_with_suffix(|suffix, window_start| {
            LazyRollingSumFromHeight::from_source(
                &format!("{name}_{suffix}"),
                version,
                cumulative,
                *window_start,
                indexes,
            )
        }))
    }
}
