use bitview_types::{DifficultyAdjustmentEntry, TimePeriod};
use vecdb::VecIndex;

use super::{epochs::difficulty_epochs, start_height};
use crate::{Query, Result, TipPin};

impl Query {
    /// The retargets whose first block lies within `time_period`'s timestamp lookback from the
    /// tip, newest first; `None` walks every epoch from genesis.
    pub fn difficulty_adjustments(
        &self,
        at: &TipPin,
        time_period: Option<TimePeriod>,
    ) -> Result<Vec<DifficultyAdjustmentEntry>> {
        let tip = at.height;
        let end = tip.to_usize();
        let start = match time_period {
            Some(tp) => start_height(self, tp, tip)?.to_usize(),
            None => 0,
        };

        let mut entries = difficulty_epochs(self.plugins(), start, end)?;
        entries.reverse();
        Ok(entries)
    }
}
