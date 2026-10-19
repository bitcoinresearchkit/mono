use std::ops::Range;

use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::{Date, Day1};
use vecdb::ReadableVec;

use crate::{Error, Query, Result, check_bound};

impl Query {
    /// The block heights `[first, end)` of calendar `day`, clamped to `end`. 404 when the day has
    /// no block below `end`; 503 while a lowered publication hides them.
    pub(super) fn day_heights(
        &self,
        day: Day1,
        end: usize,
        pin: &SafeLengths,
    ) -> Result<Range<usize>> {
        let starts = &self.plugins().mappings.day1.first_height;
        let first = starts.collect_one(day).map_or(usize::MAX, usize::from);
        let next = starts
            .collect_one_at(usize::from(day) + 1)
            .map_or(end, usize::from)
            .min(end);
        if first >= next {
            check_bound(pin, first, end)?;
            return Err(Error::NotFound(format!(
                "No published block on {}",
                Date::from(day)
            )));
        }
        Ok(first..next)
    }
}
