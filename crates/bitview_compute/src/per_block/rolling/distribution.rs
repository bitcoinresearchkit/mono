use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Database, Exit, ReadableVec, Rw, StorageMode};

use crate::{
    ComputedVecValue, DistributionStats, NumericValue, RollingWindows, WindowStarts,
    algo::compute_rolling_distribution_from_starts,
};

#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct RollingDistribution<T, M: StorageMode = Rw>(pub DistributionStats<RollingWindows<T, M>>)
where
    T: ComputedVecValue + PartialOrd + JsonSchema;

impl<T> RollingDistribution<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        Ok(Self(DistributionStats::try_from_fn(|suffix| {
            RollingWindows::forced_import(db, &format!("{name}_{suffix}"), version, indexes)
        })?))
    }

    pub fn compute_distribution(
        &mut self,
        max_from: Height,
        windows: &WindowStarts<'_>,
        source: &impl ReadableVec<Height, T>,
        exit: &Exit,
    ) -> Result<()>
    where
        T: Copy + Ord + From<f64> + Default,
        f64: From<T>,
    {
        let mut values_cache = None;
        macro_rules! compute_window {
            ($w:ident) => {
                compute_rolling_distribution_from_starts(
                    max_from,
                    windows.$w,
                    source,
                    &mut self.0.min.$w.height,
                    &mut self.0.max.$w.height,
                    &mut self.0.pct10.$w.height,
                    &mut self.0.pct25.$w.height,
                    &mut self.0.median.$w.height,
                    &mut self.0.pct75.$w.height,
                    &mut self.0.pct90.$w.height,
                    exit,
                    &mut values_cache,
                )?
            };
        }
        // Largest window first: its cache covers all smaller windows.
        compute_window!(_1y);
        compute_window!(_1m);
        compute_window!(_1w);
        compute_window!(_24h);

        Ok(())
    }
}
