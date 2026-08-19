use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Height, Lengths, StoredF32, Version};
use vecdb::{Database, Exit, ReadableVec, Rw, StorageMode};

use crate::{Lookback, PerBlock};

use super::period_suffix;

#[derive(Traversable)]
pub struct StdDevPerBlock<M: StorageMode = Rw> {
    #[traversable(skip)]
    days: usize,
    /// Arithmetic mean of the source values in the selected trailing window.
    pub sma: PerBlock<StoredF32, M>,
    /// Population standard deviation of the source values in the selected
    /// trailing window.
    pub sd: PerBlock<StoredF32, M>,
}

impl StdDevPerBlock {
    pub fn forced_import(
        db: &Database,
        name: &str,
        period: &str,
        days: usize,
        parent_version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        let version = parent_version + Version::TWO;
        let p = period_suffix(period);

        let sma = PerBlock::forced_import(db, &format!("{name}_sma{p}"), version, indexes)?;
        let sd = PerBlock::forced_import(db, &format!("{name}_sd{p}"), version, indexes)?;

        Ok(Self { days, sma, sd })
    }

    pub fn compute_all(
        &mut self,
        lookback: &impl Lookback,
        starting_lengths: &Lengths,
        exit: &Exit,
        source: &impl ReadableVec<Height, StoredF32>,
    ) -> Result<()> {
        if self.days == usize::MAX {
            self.sma.height.compute_sma_(
                starting_lengths.height,
                source,
                usize::MAX,
                exit,
                None,
            )?;
            self.sd.height.compute_expanding_sd(
                starting_lengths.height,
                source,
                &self.sma.height,
                exit,
            )?;
            return Ok(());
        }

        let window_starts = lookback.start_vec(self.days);

        self.sma.height.compute_rolling_average(
            starting_lengths.height,
            window_starts,
            source,
            exit,
        )?;

        self.sd.height.compute_rolling_sd(
            starting_lengths.height,
            window_starts,
            source,
            &self.sma.height,
            exit,
        )?;

        Ok(())
    }
}
