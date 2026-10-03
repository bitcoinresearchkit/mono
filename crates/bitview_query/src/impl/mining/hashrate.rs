use bitview_types::{DifficultyEntry, HashrateEntry, HashrateSummary, TimePeriod};
use brk_types::Height;
use vecdb::{ReadableOptionVec, ReadableVec, VecIndex};

use super::{epochs::iter_difficulty_epochs, start_height};
use crate::{Error, OptionData, Query, Result};

impl Query {
    /// Network hashrate summary for `time_period` (`None` walks the full
    /// chain). Bundles a downsampled daily hashrate series (at most
    /// `max_points.max(1)` samples), every difficulty retarget within the window, the
    /// current 1-day hashrate, and the current block's difficulty. The
    /// window cutoff uses timestamp lookbacks from the tip (via `start_height`), matching
    /// `difficulty_adjustments` so the two endpoints agree on the same
    /// `time_period`.
    pub fn hashrate(
        &self,
        time_period: Option<TimePeriod>,
        max_points: usize,
    ) -> Result<HashrateSummary> {
        let _guard = self.read_publication()?;
        let indexer = self.indexer();
        let plugins = self.plugins();
        let current_height = self.height();

        let current_difficulty = *indexer
            .vecs()
            .blocks
            .difficulty
            .collect_one(current_height)
            .data()?;

        let current_hashrate = self.hashrate_at(current_height)?;
        let current_day1 = plugins
            .mappings
            .height
            .day1
            .collect_one(current_height)
            .data()?;

        let end = current_height.to_usize();
        let start = match time_period {
            Some(tp) => start_height(self, tp)?.to_usize(),
            None => 0,
        };

        let start_day1 = plugins
            .mappings
            .height
            .day1
            .collect_one(Height::from(start))
            .data()?;
        let end_day1 = current_day1;

        // Sample at regular intervals so the chart payload stays bounded
        // regardless of window size.
        let total_days = end_day1
            .to_usize()
            .checked_sub(start_day1.to_usize())
            .ok_or(Error::Internal("Reversed hashrate day window"))?
            + 1;
        let step = total_days.div_ceil(max_points.max(1));

        let mut hr_cursor = plugins.mining.hashrate.rate.base.day1.cursor();
        let mut ts_cursor = plugins.mappings.timestamp.day1.cursor();

        let mut hashrates = Vec::with_capacity(total_days / step + 1);
        let mut di = start_day1.to_usize();
        while di <= end_day1.to_usize() {
            let hr = hr_cursor.get(di).data()?;
            let timestamp = ts_cursor.get(di).data()?;
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
            .base
            .day1
            .collect_one_flat(day)
            .data()? as u128)
    }
}
