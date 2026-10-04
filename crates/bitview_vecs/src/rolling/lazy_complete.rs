use crate::RollingTotals;
use bitview_collections::Windows;
use bitview_compute::{ComputedVecValue, NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{ReadableCloneableVec, UnaryTransform};

use crate::{IndexSources, LazyRollingDistribution, RollingComplete};

/// Lazy analog of `RollingComplete<T>`: lazy rolling sums + lazy rolling averages + lazy rolling distribution.
/// Zero stored vecs.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazyRollingComplete<T, S1T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
    S1T: ComputedVecValue + JsonSchema,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    rolling: RollingTotals<T>,
    #[traversable(flatten)]
    distribution: LazyRollingDistribution<T, S1T>,
}

impl<T, S1T> LazyRollingComplete<T, S1T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T> + 'static,
    S1T: NumericValue + JsonSchema + Quantity,
{
    pub(crate) fn from_rolling_complete<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        cumulative: &impl ReadableCloneableVec<Height, T>,
        source: &RollingComplete<S1T>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        let rolling = RollingTotals::new(name, version, cumulative, window_starts, indexes);
        let distribution = LazyRollingDistribution::from_rolling_distribution::<F>(
            name,
            version,
            &source.distribution,
        );
        Self {
            rolling,
            distribution,
        }
    }
}
