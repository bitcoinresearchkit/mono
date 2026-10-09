use bitview_collections::Windows;
use bitview_compute::ComputeRollingStats;
use bitview_primitives::{Lengths, Percent};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use rayon::prelude::*;
use vecdb::{Database, ReadableVec, Rw, StorageMode};

use crate::{IndexSources, Lookback, PerBlock};

/// Mean and population standard deviation of a per-block percentage over each trailing window.
#[derive(Traversable)]
pub struct RollingAvgSd<M: StorageMode = Rw> {
    /// Arithmetic mean of the per-block values in the trailing window.
    avg: Windows<PerBlock<Percent, M>>,
    /// Population standard deviation of the per-block values in the trailing window.
    pub sd: Windows<PerBlock<Percent, M>>,
}

impl RollingAvgSd {
    pub fn import(
        db: &Database,
        name: &str,
        parent_version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let version = parent_version + Version::new(5);
        Ok(Self {
            avg: Windows::try_from_fn(|window| {
                PerBlock::import(db, &format!("{name}_avg_{window}"), version, indexes)
            })?,
            sd: Windows::try_from_fn(|window| {
                PerBlock::import(db, &format!("{name}_sd_{window}"), version, indexes)
            })?,
        })
    }

    pub fn compute(
        &mut self,
        lookback: &(impl Lookback + Sync),
        starting_lengths: &Lengths,
        exit: &Exit,
        source: &(impl ReadableVec<Height, Percent> + Sync),
    ) -> Result<()> {
        self.avg
            .as_mut_array()
            .into_iter()
            .zip(self.sd.as_mut_array())
            .zip(Windows::<()>::DAYS)
            .collect::<Vec<_>>()
            .into_par_iter()
            .try_for_each(|((avg, sd), days)| {
                let window_starts = lookback.start_vec(days);
                avg.height.compute_rolling_average(
                    starting_lengths.height,
                    window_starts,
                    source,
                    exit,
                )?;
                sd.height.compute_rolling_sd(
                    starting_lengths.height,
                    window_starts,
                    source,
                    &avg.height,
                    exit,
                )?;
                Ok(())
            })
    }
}
