use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{LazyVec, ReadableCloneableVec, UnaryTransform, VecIndex};

use crate::{ComputedVecValue, DistributionStats, PerBlockDistribution};

/// Lazy analog of `Distribution<T>`: 7 `LazyVec` fields,
/// each derived by transforming the corresponding field of a source `PerBlockDistribution<S1T>`.
#[derive(Clone, Traversable)]
pub struct LazyDistribution<I, T, S1T>
where
    I: VecIndex,
    T: ComputedVecValue + JsonSchema,
    S1T: ComputedVecValue,
{
    /// Minimum value in the represented distribution.
    pub min: LazyVec<I, T, I, S1T>,
    /// Maximum value in the represented distribution.
    pub max: LazyVec<I, T, I, S1T>,
    /// 10th percentile of the represented distribution.
    pub pct10: LazyVec<I, T, I, S1T>,
    /// 25th percentile of the represented distribution.
    pub pct25: LazyVec<I, T, I, S1T>,
    /// Median of the represented distribution.
    pub median: LazyVec<I, T, I, S1T>,
    /// 75th percentile of the represented distribution.
    pub pct75: LazyVec<I, T, I, S1T>,
    /// 90th percentile of the represented distribution.
    pub pct90: LazyVec<I, T, I, S1T>,
}

impl<T, S1T> LazyDistribution<Height, T, S1T>
where
    T: ComputedVecValue + JsonSchema + 'static,
    S1T: ComputedVecValue + PartialOrd + JsonSchema,
{
    pub fn from_distribution<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        source: &PerBlockDistribution<S1T>,
    ) -> Self {
        let s = DistributionStats::<()>::SUFFIXES;
        Self {
            min: LazyVec::transformed::<F>(
                &format!("{name}_{}", s[0]),
                version,
                source.min.height.read_only_boxed_clone(),
            ),
            max: LazyVec::transformed::<F>(
                &format!("{name}_{}", s[1]),
                version,
                source.max.height.read_only_boxed_clone(),
            ),
            pct10: LazyVec::transformed::<F>(
                &format!("{name}_{}", s[2]),
                version,
                source.pct10.height.read_only_boxed_clone(),
            ),
            pct25: LazyVec::transformed::<F>(
                &format!("{name}_{}", s[3]),
                version,
                source.pct25.height.read_only_boxed_clone(),
            ),
            median: LazyVec::transformed::<F>(
                &format!("{name}_{}", s[4]),
                version,
                source.median.height.read_only_boxed_clone(),
            ),
            pct75: LazyVec::transformed::<F>(
                &format!("{name}_{}", s[5]),
                version,
                source.pct75.height.read_only_boxed_clone(),
            ),
            pct90: LazyVec::transformed::<F>(
                &format!("{name}_{}", s[6]),
                version,
                source.pct90.height.read_only_boxed_clone(),
            ),
        }
    }
}
