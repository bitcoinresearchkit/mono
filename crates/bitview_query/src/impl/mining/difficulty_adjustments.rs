use bitview_types::{DifficultyAdjustmentEntry, TimePeriod};
use vecdb::VecIndex;

use super::{epochs::iter_difficulty_epochs, start_height};
use crate::{Query, Result};

impl Query {
    /// All difficulty adjustments (one entry per retarget) whose first block
    /// lies within `time_period`, in reverse chronological order (newest
    /// first). `None` walks every epoch from genesis. The window cutoff is
    /// a timestamp lookback from the tip (via `start_height`) rather than block-count, so the
    /// returned set is "epochs whose first block lies within the period",
    /// not "the last N epochs".
    pub fn difficulty_adjustments(
        &self,
        time_period: Option<TimePeriod>,
    ) -> Result<Vec<DifficultyAdjustmentEntry>> {
        let pin = self.pin_safe_lengths()?;
        let tip = Query::tip_height(&pin);
        let end = tip.to_usize();
        let start = match time_period {
            Some(tp) => start_height(self, tp, tip)?.to_usize(),
            None => 0,
        };

        let mut entries = iter_difficulty_epochs(self.plugins(), start, end)?;
        drop(pin);
        entries.reverse();
        Ok(entries)
    }
}
