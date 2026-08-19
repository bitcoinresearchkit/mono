use bitview_traversable::Traversable;
use brk_types::{Height, StoredF32, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{ColumnId, PcoVec, ReadOnlyColumnarVec, ReadableCloneableVec};

use crate::{FixedRatio, LazyColumnPerBlock, LazyPerBlock, Percent};

/// Fixed-point column projection with lazy ratio and percentage views.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyColumnPercentPerBlock<B: FixedRatio, C: ColumnId>(
    pub Percent<LazyColumnPerBlock<B, C>, LazyPerBlock<StoredF32, B>>,
);

impl<B: FixedRatio, C: ColumnId> LazyColumnPercentPerBlock<B, C> {
    pub fn new(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, B>, C>,
        column: C,
        indexes: &crate::IndexSources,
    ) -> Self {
        let ppm = LazyColumnPerBlock::new(
            &format!("{name}_{}", B::SUFFIX),
            version,
            source,
            column,
            indexes,
        );
        let ratio = LazyPerBlock::from_resolutions::<B::ToRatio>(
            &format!("{name}_ratio"),
            version,
            ppm.height.read_only_boxed_clone(),
            &ppm.resolutions,
        );
        let percent = LazyPerBlock::from_resolutions::<B::ToPercent>(
            name,
            version,
            ppm.height.read_only_boxed_clone(),
            &ppm.resolutions,
        );

        Self(Percent {
            ppm,
            ratio,
            percent,
        })
    }
}
