use bitview_collections::Percent;
use bitview_compute::FixedRatio;
use bitview_traversable::Traversable;
use brk_types::{Height, StoredF32, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{DeltaRate, ReadableCloneableVec, VecValue};

use crate::{IndexSources, LazyDeltaFromHeight, LazyPerBlock};

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyDeltaPercentFromHeight<S, B>(
    pub Percent<LazyDeltaFromHeight<S, B, DeltaRate>, LazyPerBlock<StoredF32, B>>,
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
        let ppm_name = format!("{name}_rate_{}", B::SUFFIX);
        let ppm =
            LazyDeltaFromHeight::from_source(&ppm_name, version, source, window_start, indexes);

        let ratio_name = format!("{name}_rate_ratio");
        let ratio =
            LazyPerBlock::from_resolutions::<B::ToRatio>(&ratio_name, version, &ppm.resolutions);

        let percent_name = format!("{name}_rate");
        let percent = LazyPerBlock::from_resolutions::<B::ToPercent>(
            &percent_name,
            version,
            &ppm.resolutions,
        );

        Self(Percent {
            ppm,
            ratio,
            percent,
        })
    }
}
