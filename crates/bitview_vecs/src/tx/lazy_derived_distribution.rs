use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::UnaryTransform;

use crate::{LazyDistribution, TxDerivedDistribution};
use bitview_compute::ComputedVecValue;

#[derive(Clone, Traversable)]
pub struct LazyTxDerivedDistribution<T, S1T>
where
    T: ComputedVecValue + JsonSchema,
    S1T: ComputedVecValue,
{
    block: LazyDistribution<Height, T, S1T>,
    /// Uses the six-block window ending at the represented block.
    _6b: LazyDistribution<Height, T, S1T>,
}

impl<T, S1T> LazyTxDerivedDistribution<T, S1T>
where
    T: ComputedVecValue + JsonSchema + 'static,
    S1T: ComputedVecValue + PartialOrd + JsonSchema,
{
    pub(crate) fn from_tx_derived<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        source: &TxDerivedDistribution<S1T>,
    ) -> Self {
        let block = LazyDistribution::from_distribution::<F>(name, version, &source.block);
        let _6b =
            LazyDistribution::from_distribution::<F>(&format!("{name}_6b"), version, &source._6b);
        Self { block, _6b }
    }
}
