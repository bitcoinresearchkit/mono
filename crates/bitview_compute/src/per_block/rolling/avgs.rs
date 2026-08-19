use bitview_traversable::Traversable;
use brk_types::{Height, StoredF32, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{DeltaAvg, LazyDeltaVec, ReadOnlyClone, ReadableCloneableVec};

use crate::{CACHE_BUDGET, CachedWindowStartVec, NumericValue, Resolutions, Windows};

use super::LazyRollingAvgFromHeight;

/// Lazy rolling averages for all 4 window durations (24h, 1w, 1m, 1y),
/// derived from a cumulative vec + cached window starts.
///
/// Nothing is stored on disk — all values are computed on-the-fly via
/// `LazyDeltaVec<Height, T, f64, DeltaAvg>`: `(cum[h] - cum[start-1]) / (h - start + 1)`.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyRollingAvgsFromHeight<T>(
    /// Arithmetic mean of the per-block values over the trailing window ending
    /// at the represented block; each block has equal weight. At time-period
    /// indexes, the value is taken at the period's final block.
    pub Windows<LazyRollingAvgFromHeight<T>>,
)
where
    T: NumericValue + JsonSchema;

impl<T> LazyRollingAvgsFromHeight<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn new(
        name: &str,
        version: Version,
        cumulative: &(impl ReadableCloneableVec<Height, T> + 'static),
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &crate::IndexSources,
    ) -> Self {
        let cum_source = cumulative.read_only_boxed_clone();

        Self(cached_starts.map_with_suffix(|suffix, cached_start| {
            let full_name = format!("{name}_{suffix}");
            let cached = cached_start.read_only_clone();
            let starts_version = cached.version();
            let avg = LazyDeltaVec::<Height, T, StoredF32, DeltaAvg>::new(
                &full_name,
                version,
                cum_source.clone(),
                starts_version,
                move || cached.snapshot(),
            );
            let source = CACHE_BUDGET.wrap(avg.clone());
            let resolutions = Resolutions::from_height_source(&full_name, source, version, indexes);
            LazyRollingAvgFromHeight {
                height: avg,
                resolutions: Box::new(resolutions),
            }
        }))
    }
}
