use bitview_traversable::Traversable;
use brk_types::{TxIndex, Version};
use schemars::JsonSchema;
use vecdb::{LazyVec, UnaryTransform};

use crate::{ComputedVecValue, LazyTxDerivedDistribution, TxDerivedDistribution};

/// Per-transaction lazy values with a distribution transformed from another type's distribution.
#[derive(Clone, Traversable)]
pub struct LazyPerTxDistributionTransformed<T, S, DSource>
where
    T: ComputedVecValue + JsonSchema,
    S: ComputedVecValue,
    DSource: ComputedVecValue,
{
    pub tx_index: LazyVec<TxIndex, T, TxIndex, S>,
    #[traversable(flatten)]
    pub distribution: LazyTxDerivedDistribution<T, DSource>,
}

impl<T, S, DSource> LazyPerTxDistributionTransformed<T, S, DSource>
where
    T: ComputedVecValue + JsonSchema + 'static,
    S: ComputedVecValue + JsonSchema,
    DSource: ComputedVecValue + JsonSchema,
{
    pub fn new<F: UnaryTransform<DSource, T>>(
        name: &str,
        version: Version,
        tx_index: LazyVec<TxIndex, T, TxIndex, S>,
        source_distribution: &TxDerivedDistribution<DSource>,
    ) -> Self {
        let distribution =
            LazyTxDerivedDistribution::from_tx_derived::<F>(name, version, source_distribution);
        Self {
            tx_index,
            distribution,
        }
    }
}
