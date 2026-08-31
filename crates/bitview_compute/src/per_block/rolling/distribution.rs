use brk_error::Result;

use bitview_traversable::Traversable;
use brk_exit::Exit;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use schemars::JsonSchema;
use vecdb::{Database, ReadableVec, Rw, StorageMode};

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
        let DistributionStats {
            min,
            max,
            pct10,
            pct25,
            median,
            pct75,
            pct90,
        } = &mut self.0;
        let min = &mut min.0;
        let max = &mut max.0;
        let pct10 = &mut pct10.0;
        let pct25 = &mut pct25.0;
        let median = &mut median.0;
        let pct75 = &mut pct75.0;
        let pct90 = &mut pct90.0;

        macro_rules! window {
            ($w:ident) => {
                DistributionStats {
                    min: &mut min.$w.height,
                    max: &mut max.$w.height,
                    pct10: &mut pct10.$w.height,
                    pct25: &mut pct25.$w.height,
                    median: &mut median.$w.height,
                    pct75: &mut pct75.$w.height,
                    pct90: &mut pct90.$w.height,
                }
            };
        }

        let year = window!(_1y);
        let month = window!(_1m);
        let week = window!(_1w);
        let day = window!(_24h);

        [
            (year, windows._1y),
            (month, windows._1m),
            (week, windows._1w),
            (day, windows._24h),
        ]
        .into_par_iter()
        .try_for_each(|(outputs, starts)| {
            let mut values_cache = None;
            compute_rolling_distribution_from_starts(
                max_from,
                starts,
                source,
                outputs,
                exit,
                &mut values_cache,
            )
        })
    }
}
