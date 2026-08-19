use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{ReadableCloneableVec, UnaryTransform};

use crate::{
    CachedWindowStartVec, ComputedVecValue, LazyRollingAvgsFromHeight, LazyRollingDistribution,
    LazyRollingSumsFromHeight, NumericValue, RollingComplete, Windows,
};

/// Lazy analog of `RollingComplete<T>`: lazy rolling sums + lazy rolling averages + lazy rolling distribution.
/// Zero stored vecs.
#[derive(Clone, Traversable)]
pub struct LazyRollingComplete<T, S1T>
where
    T: NumericValue + JsonSchema,
    S1T: ComputedVecValue + JsonSchema,
{
    pub sum: LazyRollingSumsFromHeight<T>,
    pub average: LazyRollingAvgsFromHeight<T>,
    #[traversable(flatten)]
    pub distribution: LazyRollingDistribution<T, S1T>,
}

impl<T, S1T> LazyRollingComplete<T, S1T>
where
    T: NumericValue + JsonSchema + 'static,
    S1T: NumericValue + JsonSchema,
{
    pub fn from_rolling_complete<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        cumulative: &(impl ReadableCloneableVec<Height, T> + 'static),
        source: &RollingComplete<S1T>,
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &crate::IndexSources,
    ) -> Self {
        let sum = LazyRollingSumsFromHeight::new(
            &format!("{name}_sum"),
            version,
            cumulative,
            cached_starts,
            indexes,
        );
        let average = LazyRollingAvgsFromHeight::new(
            &format!("{name}_average"),
            version,
            cumulative,
            cached_starts,
            indexes,
        );
        let distribution = LazyRollingDistribution::from_rolling_distribution::<F>(
            name,
            version,
            &source.distribution,
        );
        Self {
            sum,
            average,
            distribution,
        }
    }
}
