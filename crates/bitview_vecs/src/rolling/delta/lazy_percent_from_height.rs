use bitview_collections::PercentViews;
use bitview_compute::FixedRatio;
use bitview_primitives::Percent;
use bitview_transforms::FixedToPercent;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{DeltaRate, ReadableCloneableVec, VecValue};

use crate::{IndexSources, LazyDeltaFromHeight, LazyPerBlock};

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyDeltaPercentFromHeight<S, B>(
    pub PercentViews<LazyDeltaFromHeight<S, B, DeltaRate>, LazyPerBlock<Percent, B>>,
)
where
    S: VecValue,
    B: FixedRatio;

impl<S, B> LazyDeltaPercentFromHeight<S, B>
where
    S: VecValue + Into<f64>,
    B: FixedRatio + From<f64>,
{
    pub(crate) fn from_source(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        window_start: &impl ReadableCloneableVec<Height, Height>,
        indexes: &IndexSources,
    ) -> Self {
        let fixed_name = format!("{name}_rate_{}", B::SUFFIX);
        let fixed =
            LazyDeltaFromHeight::from_source(&fixed_name, version, source, window_start, indexes);

        let percent_name = format!("{name}_rate");
        let percent = LazyPerBlock::from_resolutions::<FixedToPercent>(
            &percent_name,
            version,
            &fixed.resolutions,
        );

        Self(PercentViews { fixed, percent })
    }
}
