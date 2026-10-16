use bitview_types::{DifficultyEntry, HashrateEntry, HashrateSummary, TimePeriod};
use brk_types::{Height, TARGET_BLOCKS_PER_DAY_F64};
use vecdb::{ReadableVec, VecIndex};

use super::{epochs::iter_difficulty_epochs, start_height};
use crate::{Error, OptionData, Query, Result};

impl Query {
    /// Network hashrate summary for `time_period` (`None` walks the full
    /// chain). Bundles a downsampled series of completed days' hashrates (at most
    /// `max_points.max(1)` samples), each stamped at its day's end as mempool.space does, every
    /// difficulty retarget within the window, the trailing 7-day hashrate (the blocks of the
    /// last 7 days), and the current block's difficulty. The
    /// window cutoff uses timestamp lookbacks from the tip (via `start_height`), matching
    /// `difficulty_adjustments` so the two endpoints agree on the same
    /// `time_period`.
    pub fn hashrate(
        &self,
        time_period: Option<TimePeriod>,
        max_points: usize,
    ) -> Result<HashrateSummary> {
        let pin = self.pin_safe_lengths()?;
        let tip = Query::tip_height(&pin)?;
        let indexer = self.indexer();
        let plugins = self.plugins();
        let current_height = tip;

        let current_difficulty = *indexer
            .vecs()
            .blocks
            .difficulty
            .collect_one(current_height)
            .data()?;

        let current_hashrate = self.trailing_hashrate(current_height, 7)?;
        let current_day1 = plugins
            .mappings
            .height
            .day1
            .collect_one(current_height)
            .data()?;

        let end = current_height.to_usize();
        let start = match time_period {
            Some(tp) => start_height(self, tp, tip)?.to_usize(),
            None => 0,
        };

        let start_day1 = plugins
            .mappings
            .height
            .day1
            .collect_one(Height::from(start))
            .data()?;
        let end_day1 = current_day1;

        // Completed days only (the tip's is still open), sampled at regular intervals so the
        // chart payload stays bounded regardless of window size.
        let days = end_day1
            .to_usize()
            .checked_sub(start_day1.to_usize())
            .ok_or(Error::Internal("Reversed hashrate day window"))?;
        let step = days.div_ceil(max_points.max(1)).max(1);

        let mut hr_cursor = plugins.mining.hashrate.rate.block.day1.cursor();
        let mut ts_cursor = plugins.mappings.timestamp.day1.cursor();

        let mut hashrates = Vec::with_capacity(days.div_ceil(step));
        let mut di = start_day1.to_usize();
        while di < end_day1.to_usize() {
            let hr = hr_cursor.get(di).data()?;
            // The day's end: the next day's start.
            let timestamp = ts_cursor.get(di + 1).data()?;
            if let Some(hr) = hr {
                hashrates.push(HashrateEntry {
                    timestamp,
                    avg_hashrate: *hr as u128,
                });
            }
            di += step;
        }

        let difficulty: Vec<DifficultyEntry> = iter_difficulty_epochs(plugins, start, end)?
            .into_iter()
            .map(|e| DifficultyEntry {
                time: e.timestamp,
                height: e.height,
                difficulty: e.difficulty,
                adjustment: e.change_percent,
            })
            .collect();

        Ok(HashrateSummary {
            hashrates,
            difficulty,
            current_hashrate,
            current_difficulty,
        })
    }

    /// Network 1-day hashrate at the day containing `height`. Errors on
    /// stamp lag in the day1 index or in the daily-hashrate vec, so a
    /// transient dropout surfaces instead of silently reporting zero.
    pub(crate) fn hashrate_at(&self, height: Height) -> Result<u128> {
        let plugins = self.plugins();
        let day = plugins.mappings.height.day1.collect_one(height).data()?;
        Ok(*plugins
            .mining
            .hashrate
            .rate
            .block
            .day1
            .collect_one(day)
            .flatten()
            .data()? as u128)
    }

    /// Network hashrate over the `days` days ending at `height`: the work its blocks proved per
    /// second of the window, as the one-day [`Self::hashrate_at`] measures it.
    pub(crate) fn trailing_hashrate(&self, height: Height, days: usize) -> Result<u128> {
        let plugins = self.plugins();
        let start = plugins
            .mappings
            .lookback
            .start_vec(days)
            .collect_one(height)
            .data()?;
        let mut daily_work = 0.0;
        plugins
            .blocks
            .difficulty
            .hashrate
            .height
            .for_each_range_dyn(start, height.incremented(), &mut |hashrate| {
                daily_work += f64::from(hashrate)
            });
        Ok((daily_work / (days as f64 * TARGET_BLOCKS_PER_DAY_F64)) as u128)
    }
}
