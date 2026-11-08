use std::ops::Range;

use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::{Date, Day1};
use vecdb::ReadableVec;

use crate::{Error, Query, Result, check_bound};

impl Query {
    /// The block heights `[first, end)` of calendar `day`, clamped to `end`, and whether the day
    /// is settled: it ends within `end` and the next day's first block, which ends it, lies below
    /// the reorg window. 404 when the day has no block below `end`; 503 while a lowered
    /// publication hides them.
    pub(super) fn day_heights(
        &self,
        day: Day1,
        end: usize,
        pin: &SafeLengths,
    ) -> Result<(Range<usize>, bool)> {
        let starts = &self.plugins().mappings.day1.first_height;
        let first = starts.collect_one(day).map_or(usize::MAX, usize::from);
        let boundary = starts.collect_one_at(usize::from(day) + 1);
        let next = boundary.map_or(end, usize::from).min(end);
        let settled = boundary.is_some_and(|boundary| {
            usize::from(boundary) <= end && pin.lengths().is_settled(boundary)
        });
        if first >= next {
            check_bound(pin, first, end)?;
            return Err(Error::NotFound(format!(
                "No published block on {}",
                Date::from(day)
            )));
        }
        Ok((first..next, settled))
    }
}
