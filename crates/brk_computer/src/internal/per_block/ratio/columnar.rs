use brk_traversable::Traversable;
use brk_types::{Height, StoredF32, Version};
use vecdb::{ColumnId, PcoVec, ReadOnlyColumnarVec, ReadableCloneableVec};

use crate::{
    indexes,
    internal::{FixedRatio, LazyColumnPerBlock, LazyPerBlock},
};

#[derive(Clone, Traversable)]
pub struct LazyColumnRatioPerBlock<R, C>
where
    R: FixedRatio,
    C: ColumnId,
{
    pub ppm: LazyColumnPerBlock<R, C>,
    pub ratio: LazyPerBlock<StoredF32, R>,
}

impl<R, C> LazyColumnRatioPerBlock<R, C>
where
    R: FixedRatio,
    C: ColumnId,
{
    pub(crate) fn new(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, R>, C>,
        column: C,
        indexes: &indexes::Vecs,
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
