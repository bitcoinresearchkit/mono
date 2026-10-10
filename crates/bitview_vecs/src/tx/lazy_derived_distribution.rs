use bitview_collections::DistributionStats;
use bitview_compute::ComputedVecValue;
use bitview_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::UnaryTransform;

use crate::{LazyPerBlock, TxDerivedDistribution, TxWindows};

/// Lazy analog of `TxDerivedDistribution`: each statistic of each window transformed from the
/// source's.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyTxDerivedDistribution<T, S1T>(DistributionStats<TxWindows<LazyPerBlock<T, S1T>>>)
where
    T: ComputedVecValue + JsonSchema,
    S1T: ComputedVecValue;

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
        Self(source.0.map_with_suffix(|stat, source| TxWindows {
            block: LazyPerBlock::from_resolutions::<F>(
                &format!("{name}_{stat}"),
                version,
                &source.block.resolutions,
            ),
            _6b: LazyPerBlock::from_resolutions::<F>(
                &format!("{name}_{stat}_6b"),
                version,
                &source._6b.resolutions,
            ),
        }))
    }
}
