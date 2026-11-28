//! Day-level series: one row a day, read at the day's close (its last block, UTC), the current day
//! rewritten as its blocks arrive.

use std::ops::Range;

use bitview_primitives::Day1;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use vecdb::{AnyStoredVec, ReadableVec, VecIndex};

use crate::prepare_computed;

/// [`prepare_computed`] for day-level outputs, once the blocks from `from` on may have changed,
/// returning the first day to write. That is the day of the block before `from` (the first block a
/// rewrite replaces can fall on a later day than the one it removed), at the latest the last day
/// written among the `days` the inputs cover: it was written open, and a stop between the inputs
/// crossing its midnight and this update would otherwise leave it at an older tip.
pub fn prepare_days<'a, V: AnyStoredVec + ?Sized + 'a>(
    mut outputs: impl AsMut<[&'a mut V]>,
    version: Version,
    day_of_height: &impl ReadableVec<Height, Day1>,
    from: Height,
    days: usize,
    exit: &Exit,
) -> Result<usize> {
    let written = outputs.as_mut().iter().map(|vec| vec.len()).min().unwrap_or(0);
    let before = from.to_usize().saturating_sub(1);
    let from_day = day_of_height
        .collect_one_at(before.min(day_of_height.len().saturating_sub(1)))
        .map_or(0, |day| day.to_usize())
        .min(days.min(written).saturating_sub(1));
    prepare_computed(outputs, version, from_day, exit)
}

/// Each of `days`' close among the first `blocks` blocks, its last one, from each day's first
/// height; `None` for a day without blocks (January 2009).
pub fn day_closes(
    first_height: &impl ReadableVec<Day1, Height>,
    days: Range<usize>,
    blocks: usize,
) -> Vec<Option<usize>> {
    let start_of = |day: usize| {
        first_height
            .collect_one_at(day)
            .map_or(blocks, |height| height.to_usize())
            .min(blocks)
    };
    let mut start = start_of(days.start);
    days.map(|day| {
        let next = start_of(day + 1);
        let close = (next > start).then(|| next - 1);
        start = next;
        close
    })
    .collect()
}
