use bitview_collections::FixedRatioViews;
use bitview_compute::FixedRatio;
use bitview_primitives::{Percent, Ratio};
use bitview_transforms::{FixedToPercent, FixedToRatio};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{LazyVec, ReadableCloneableVec, VecValue};

/// Fully lazy lightweight percent container with no derived resolutions.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyFixedRatioVec<B: FixedRatio, S: VecValue>(
    pub  FixedRatioViews<
        LazyVec<Height, B, Height, S>,
        LazyVec<Height, Ratio, Height, B>,
        LazyVec<Height, Percent, Height, B>,
    >,
);

impl<B: FixedRatio, S: VecValue> LazyFixedRatioVec<B, S> {
    pub fn from_indexed_source(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        compute: fn(Height, S) -> B,
    ) -> Self {
        let ppm = LazyVec::init(
            &format!("{name}_{}", B::SUFFIX),
            version,
            source.read_only_boxed_clone(),
            compute,
        );
        let ppm_source = ppm.read_only_boxed_clone();
        let ratio = LazyVec::transformed::<FixedToRatio>(
            &format!("{name}_ratio"),
            version,
            ppm_source.clone(),
        );
        let percent = LazyVec::transformed::<FixedToPercent>(name, version, ppm_source);

        Self(FixedRatioViews {
            ppm,
            ratio,
            percent,
        })
    }
}
