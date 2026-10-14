use bitview_compute::{
    ComputedVecValue, DistributionStats, NumericValue, Retained, SortedWindow,
    compute_rolling_distribution_from_starts,
};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use schemars::JsonSchema;
use vecdb::{Budgeted, Database, EagerVec, PcoVec, ReadableVec, Rw, StorageMode, VecValue};

use crate::{IndexSources, RollingWindows, WindowStarts};

#[derive(Deref, DerefMut, Traversable)]
pub struct RollingDistribution<T, M: StorageMode = Rw>
where
    T: ComputedVecValue + PartialOrd + JsonSchema,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    stats: DistributionStats<RollingWindows<T, M>>,
    /// The kept window of each rolling window, in `Windows` order.
    retained: M::WriteOnly<[Retained<SortedWindow>; 4]>,
}

impl<T> RollingDistribution<T>
where
    T: NumericValue + JsonSchema,
{
    pub(crate) fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self {
            stats: DistributionStats::try_from_fn(|suffix| {
                RollingWindows::import(db, &format!("{name}_{suffix}"), version, indexes)
            })?,
            retained: Default::default(),
        })
    }

    pub(crate) fn compute_distribution<S>(
        &mut self,
        max_from: Height,
        windows: &WindowStarts<'_>,
        source: &impl ReadableVec<Height, S>,
        exit: &Exit,
    ) -> Result<()>
    where
        T: Copy + Ord + From<f64> + Default,
        S: VecValue + Copy,
        f64: From<S>,
    {
        let DistributionStats {
            min,
            max,
            pct10,
            pct25,
            median,
            pct75,
            pct90,
        } = &mut self.stats;
        let min = &mut min.0;
        let max = &mut max.0;
        let pct10 = &mut pct10.0;
        let pct25 = &mut pct25.0;
        let median = &mut median.0;
        let pct75 = &mut pct75.0;
        let pct90 = &mut pct90.0;

        macro_rules! window {
            ($w:ident) => {
                DistributionStats::<&mut EagerVec<PcoVec<Height, T, Budgeted>>> {
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

        let [day_kept, week_kept, month_kept, year_kept] = &mut self.retained;

        [
            (year, windows._1y, year_kept),
            (month, windows._1m, month_kept),
            (week, windows._1w, week_kept),
            (day, windows._24h, day_kept),
        ]
        .into_par_iter()
        .try_for_each(|(outputs, starts, retained)| {
            compute_rolling_distribution_from_starts(
                max_from, starts, source, outputs, retained, exit,
            )
        })
    }
}
