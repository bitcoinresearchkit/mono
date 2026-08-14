use brk_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{DeltaSub, LazyDeltaVec, ReadOnlyClone, ReadableCloneableVec};

use crate::{
    indexes,
    internal::{CachedWindowStartVec, NumericValue, Resolutions, Windows},
};

use super::LazyRollingSumFromHeight;

/// Lazy rolling sums for all 4 window durations (24h, 1w, 1m, 1y),
/// derived from a cumulative vec + cached window starts.
///
/// Nothing is stored on disk — all values are computed on-the-fly via
/// `LazyDeltaVec<Height, T, T, DeltaSub>`: `cum[h] - cum[window_start[h]]`.
///
/// Implements `Traversable` to expose `_24h`, `_1w`, `_1m`, `_1y` with
/// the same tree structure as the old `RollingWindows<T>`.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyRollingSumsFromHeight<T>(pub Windows<LazyRollingSumFromHeight<T>>)
where
    T: NumericValue + JsonSchema;

impl<T> LazyRollingSumsFromHeight<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn new(
        name: &str,
        version: Version,
        cumulative: &(impl ReadableCloneableVec<Height, T> + 'static),
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &indexes::Vecs,
    ) -> Self {
        let cum_source = cumulative.read_only_boxed_clone();

        Self(cached_starts.map_with_suffix(|suffix, cached_start| {
            let full_name = format!("{name}_{suffix}");
            let cached = cached_start.read_only_clone();
            let starts_version = cached.version();
            let sum = LazyDeltaVec::<Height, T, T, DeltaSub>::new(
                &full_name,
                version,
                cum_source.clone(),
                starts_version,
                move || cached.snapshot(),
            );
            let resolutions = Resolutions::forced_import(&full_name, sum.clone(), version, indexes);
            LazyRollingSumFromHeight {
                height: sum,
                resolutions: Box::new(resolutions),
            }
        }))
    }

    /// Build rolling sums from compact in-memory state without adding the four
    /// full-height rolling vecs to `cache_budget`.
    pub fn new_uncached(
        name: &str,
        version: Version,
        cumulative: &(impl ReadableCloneableVec<Height, T> + 'static),
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &indexes::Vecs,
    ) -> Self {
        let cum_source = cumulative.read_only_boxed_clone();

        Self(cached_starts.map_with_suffix(|suffix, cached_start| {
            let full_name = format!("{name}_{suffix}");
            let cached = cached_start.read_only_clone();
            let starts_version = cached.version();
            let sum = LazyDeltaVec::<Height, T, T, DeltaSub>::new(
                &full_name,
                version,
                cum_source.clone(),
                starts_version,
                move || cached.snapshot(),
            );
            let resolutions =
                Resolutions::forced_import_uncached(&full_name, sum.clone(), version, indexes);
            LazyRollingSumFromHeight {
                height: sum,
                resolutions: Box::new(resolutions),
            }
        }))
    }
}
