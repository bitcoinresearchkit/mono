use brk_types::{DifficultyAdjustmentEntry, TimePeriod};
use vecdb::VecIndex;

use super::epochs::iter_difficulty_epochs;
use crate::Query;

impl Query {
    /// All difficulty adjustments (one entry per retarget) whose first block
    /// lies within `time_period`, in reverse chronological order (newest
    /// first). `None` walks every epoch from genesis. The window cutoff is
    /// wall-clock (via `start_height`) rather than block-count, so the
    /// returned set is "epochs whose first block lies within the period",
    /// not "the last N epochs".
    pub fn difficulty_adjustments(
        &self,
        time_period: Option<TimePeriod>,
    ) -> brk_error::Result<Vec<DifficultyAdjustmentEntry>> {
        let end = self.height().to_usize();
        let start = match time_period {
            Some(tp) => super::start_height(self, tp)?.to_usize(),
            None => 0,
        };

        let mut entries = iter_difficulty_epochs(self.plugins(), start, end)?;
        entries.reverse();
        Ok(entries)
    }
}
