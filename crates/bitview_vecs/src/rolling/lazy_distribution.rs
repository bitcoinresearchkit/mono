use bitview_collections::{DistributionStats, Windows};
use bitview_compute::{ComputedVecValue, NumericValue};
use bitview_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::UnaryTransform;

use crate::{LazyPerBlock, RollingDistribution};

/// Lazy analog of `RollingDistribution<T>`: `DistributionStats<Windows<LazyPerBlock<T, S1T>>>`.
/// 7 stats × 4 windows = 28 lazy vecs, zero stored.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyRollingDistribution<T, S1T>(pub DistributionStats<Windows<LazyPerBlock<T, S1T>>>)
where
    T: ComputedVecValue + PartialOrd + JsonSchema,
    S1T: ComputedVecValue + JsonSchema;

impl<T, S1T> LazyRollingDistribution<T, S1T>
where
    T: ComputedVecValue + JsonSchema + 'static,
    S1T: NumericValue + JsonSchema,
{
    pub(crate) fn from_rolling_distribution<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        source: &RollingDistribution<S1T>,
    ) -> Self {
        Self(source.0.map_with_suffix(|stat, source| {
            source.0.map_with_suffix(|window, source| {
                LazyPerBlock::from_resolutions::<F>(
                    &format!("{name}_{stat}_{window}"),
                    version,
                    source,
                )
            })
        }))
    }
}
