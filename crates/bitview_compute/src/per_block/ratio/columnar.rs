use bitview_traversable::Traversable;
use brk_types::{Height, StoredF32, Version};
use vecdb::{ColumnId, PcoVec, ReadOnlyColumnarVec, ReadableCloneableVec};

use crate::{FixedRatio, LazyColumnPerBlock, LazyPerBlock};

#[derive(Clone, Traversable)]
pub struct LazyColumnRatioPerBlock<R, C>
where
    R: FixedRatio,
    C: ColumnId,
{
    /// Unitless ratio in parts per million; 1,000,000 represents 1.0.
    pub ppm: LazyColumnPerBlock<R, C>,
    /// Unitless decimal ratio derived as parts per million divided by 1,000,000.
    pub ratio: LazyPerBlock<StoredF32, R>,
}

impl<R, C> LazyColumnRatioPerBlock<R, C>
where
    R: FixedRatio,
    C: ColumnId,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, R>, C>,
        column: C,
        indexes: &crate::IndexSources,
    ) -> Self {
        let ppm = LazyColumnPerBlock::new(
            &format!("{name}_{}", R::SUFFIX),
            version,
            source,
            column,
            indexes,
        );
        let ratio = LazyPerBlock::from_resolutions::<R::ToRatio>(
            name,
            version,
            ppm.height.read_only_boxed_clone(),
            &ppm.resolutions,
        );

        Self { ppm, ratio }
    }
}
