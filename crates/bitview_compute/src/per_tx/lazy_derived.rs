use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::UnaryTransform;

use crate::{ComputedVecValue, LazyDistribution, TxDerivedDistribution};

#[derive(Clone, Traversable)]
pub struct LazyBlockRollingDistribution<T, S1T>
where
    T: ComputedVecValue + JsonSchema,
    S1T: ComputedVecValue,
{
    pub _6b: LazyDistribution<Height, T, S1T>,
}

#[derive(Clone, Traversable)]
pub struct LazyTxDerivedDistribution<T, S1T>
where
    T: ComputedVecValue + JsonSchema,
    S1T: ComputedVecValue,
{
    pub block: LazyDistribution<Height, T, S1T>,
    #[traversable(flatten)]
    pub distribution: LazyBlockRollingDistribution<T, S1T>,
}

impl<T, S1T> LazyTxDerivedDistribution<T, S1T>
where
    T: ComputedVecValue + JsonSchema + 'static,
    S1T: ComputedVecValue + PartialOrd + JsonSchema,
{
    pub fn from_tx_derived<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        source: &TxDerivedDistribution<S1T>,
    ) -> Self {
        let block = LazyDistribution::from_distribution::<F>(name, version, &source.block);
        let distribution = LazyBlockRollingDistribution {
            _6b: LazyDistribution::from_distribution::<F>(
                &format!("{name}_6b"),
                version,
                &source.distribution._6b,
            ),
        };
        Self {
            block,
            distribution,
        }
    }
}
