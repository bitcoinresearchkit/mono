use bitview_collections::DistributionStats;
use bitview_compute::ComputedVecValue;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{LazyVec, UnaryTransform, VecIndex};

use crate::PerBlockDistribution;

/// Lazy analog of `Distribution<T>`: 7 `LazyVec` fields,
/// each derived by transforming the corresponding field of a source `PerBlockDistribution<S1T>`.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyDistribution<I, T, S1T>(pub DistributionStats<LazyVec<I, T, I, S1T>>)
where
    I: VecIndex,
    T: ComputedVecValue + JsonSchema,
    S1T: ComputedVecValue;

impl<T, S1T> LazyDistribution<Height, T, S1T>
where
    T: ComputedVecValue + JsonSchema + 'static,
    S1T: ComputedVecValue + PartialOrd + JsonSchema,
{
    pub(crate) fn from_distribution<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        source: &PerBlockDistribution<S1T>,
    ) -> Self {
        Self(source.0.map_with_suffix(|suffix, source| {
            LazyVec::transformed::<F>(
                &format!("{name}_{suffix}"),
                version,
                source.height.read_only_boxed_clone(),
            )
        }))
    }
}
use vecdb::ReadableCloneableVec;
