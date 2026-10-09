use bitview_collections::Windows;
use bitview_compute::{FixedRatio, NumericValue};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{BinaryTransform, ReadableCloneableVec, ReverseOperands};

use crate::{IndexSources, LazyRatioPerBlock, LazyRollingRatioVec};

/// Fully lazy rolling ratio windows — 4 windows (24h, 1w, 1m, 1y), each with
/// lazy fixed-point values and a lazy ratio view.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyRatioRollingWindows<R: FixedRatio>(pub Windows<LazyRatioPerBlock<R>>);

impl<R: FixedRatio> LazyRatioRollingWindows<R> {
    /// The numerator's window total over the denominator's, from two cumulative sources;
    /// the denominator is read as the windowed source and the numerator as its operand.
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
        F: BinaryTransform<S, D, R> + Send + Sync + 'static,
    {
        Self(window_starts.map_with_suffix(|suffix, window_start| {
            let full_name = format!("{name}_{suffix}");
            let ratio = LazyRollingRatioVec::<D, S, R, ReverseOperands<F>>::new(
                &format!("{full_name}_{}_source", R::SUFFIX),
                version,
                denominator,
                numerator,
                *window_start,
            );
            LazyRatioPerBlock::from_height_source(&full_name, version, &ratio, indexes)
        }))
    }
}
