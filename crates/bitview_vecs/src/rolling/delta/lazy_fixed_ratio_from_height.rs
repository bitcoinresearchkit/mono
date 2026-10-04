use bitview_collections::FixedRatioViews;
use bitview_compute::FixedRatio;
use bitview_primitives::{Percent, Ratio};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{DeltaRate, ReadableCloneableVec, VecValue};

use crate::{IndexSources, LazyDeltaFromHeight, LazyPerBlock};

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyDeltaFixedRatioFromHeight<S, B>(
    pub  FixedRatioViews<
        LazyDeltaFromHeight<S, B, DeltaRate>,
        LazyPerBlock<Ratio, B>,
        LazyPerBlock<Percent, B>,
    >,
)
where
    S: VecValue,
    B: FixedRatio;

impl<S, B> LazyDeltaFixedRatioFromHeight<S, B>
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

        Self(FixedRatioViews {
            ppm,
            ratio,
            percent,
        })
    }
}
