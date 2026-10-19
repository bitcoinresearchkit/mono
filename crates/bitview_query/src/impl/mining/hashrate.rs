use bitview_types::{DifficultyEntry, HashrateEntry, HashrateSummary, TimePeriod};
use brk_types::{Height, TARGET_BLOCKS_PER_DAY_F64};
use vecdb::{ReadableVec, VecIndex};

use super::{epochs::difficulty_epochs, start_height};
use crate::{Error, OptionData, Query, Result};

impl Query {
    /// Network hashrate summary over `time_period`'s timestamp lookback from the tip (`None`: the
    /// full chain): completed days' hashrates, each stamped at its day's end as mempool.space
    /// does and sampled down to at most `max_points.max(1)`; the window's retargets; the
    /// trailing 7-day hashrate; and the tip's difficulty.
    pub fn hashrate(
        &self,
        time_period: Option<TimePeriod>,
        max_points: usize,
    ) -> Result<HashrateSummary> {
        let pin = self.pin_safe_lengths()?;
        let tip = Query::tip_height(&pin)?;
        let indexer = self.indexer();
        let plugins = self.plugins();

        let current_difficulty = *indexer.vecs().blocks.difficulty.collect_one(tip).data()?;

        let current_hashrate = self.trailing_hashrate(tip, 7)?;
        let end_day1 = plugins.mappings.height.day1.collect_one(tip).data()?;

        let end = tip.to_usize();
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
        for di in (start_day1.to_usize()..end_day1.to_usize()).step_by(step) {
            let hr = hr_cursor.get(di).data()?;
            // The day's end: the next day's start.
            let timestamp = ts_cursor.get(di + 1).data()?;
            if let Some(hr) = hr {
                hashrates.push(HashrateEntry {
                    timestamp,
                    avg_hashrate: *hr as u128,
                });
            }
        }

        let difficulty: Vec<DifficultyEntry> = difficulty_epochs(plugins, start, end)?
            .into_iter()
            .map(|e| DifficultyEntry {
                time: e.timestamp,
                height: e.height,
                difficulty: e.difficulty,
                adjustment: e.adjustment,
            })
            .collect();

        Ok(HashrateSummary {
            hashrates,
            difficulty,
            current_hashrate,
            current_difficulty,
        })
    }

    /// Network hashrate of the day containing `height`; `Internal` when it is missing or
    /// undefined.
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
