use bitview_compute::{FixedRatio, NumericValue};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{BinaryTransform, ReadableCloneableVec};

use crate::{IndexSources, LazyRatioPerBlock, LazyRollingRatioVec, shapes::Windows};

/// Fully lazy rolling ratio windows — 4 windows (24h, 1w, 1m, 1y), each with
/// lazy fixed-point values and a lazy ratio view.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazyRatioRollingWindows<R: FixedRatio>(pub Windows<LazyRatioPerBlock<R>>);

impl<R: FixedRatio> LazyRatioRollingWindows<R> {
    /// `F(source, operand)` of two cumulative sources' window totals; `source` is read first.
    pub fn from_cumulative_ratio<S, D, F>(
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
        F: BinaryTransform<S, D, R> + Send + Sync + 'static,
    {
        Self(window_starts.map_with_suffix(|suffix, window_start| {
            let full_name = format!("{name}_{suffix}");
            let ratio = LazyRollingRatioVec::<S, D, R, F>::new(
                &format!("{full_name}_{}_source", R::SUFFIX),
                version,
                source,
                operand,
                *window_start,
            );
            LazyRatioPerBlock::from_height_source(&full_name, version, &ratio, indexes)
        }))
    }
}
