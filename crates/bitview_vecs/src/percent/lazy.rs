use bitview_collections::PercentViews;
use bitview_compute::FixedRatio;
use bitview_primitives::Percent;
use bitview_transforms::FixedToPercent;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{LazyVec, ReadableCloneableVec, VecValue};

/// Fully lazy lightweight percent container with no derived resolutions.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyPercentVec<B: FixedRatio, S: VecValue>(
    pub PercentViews<LazyVec<Height, B, Height, S>, LazyVec<Height, Percent, Height, B>>,
);

impl<B: FixedRatio, S: VecValue> LazyPercentVec<B, S> {
    pub fn from_indexed_source(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        compute: fn(Height, S) -> B,
    ) -> Self {
        let fixed = LazyVec::init(
            &format!("{name}_{}", B::SUFFIX),
            version,
            source.read_only_boxed_clone(),
            compute,
        );
        let percent =
            LazyVec::transformed::<FixedToPercent>(name, version, fixed.read_only_boxed_clone());

        Self(PercentViews { fixed, percent })
    }
}
