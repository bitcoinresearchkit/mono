use std::time::{SystemTime, UNIX_EPOCH};

use brk_error::OptionData;
use brk_types::{DifficultyAdjustment, Epoch, Height};
use vecdb::ReadableVec;

use crate::Query;

/// Blocks per difficulty epoch (2 weeks target)
const BLOCKS_PER_EPOCH: u32 = 2016;

/// Target block time in seconds (10 minutes)
const TARGET_BLOCK_TIME: u64 = 600;

impl Query {
    /// Live difficulty-adjustment snapshot for the current epoch. Bundles
    /// progress through the 2016-block window, the projected next-retarget
    /// percentage from observed pace, an estimated wall-clock retarget time,
    /// remaining blocks/time, the previous retarget percentage (current epoch
    /// vs previous epoch first-block difficulty), and the time offset from a
    /// 600s/block schedule. Output time fields are in milliseconds.
    pub fn difficulty_adjustment(&self) -> brk_error::Result<DifficultyAdjustment> {
        let indexer = self.indexer();
        let computer = self.computer();
        let current_height = self.height();
        let current_height_u32: u32 = current_height.into();

        let current_epoch = computer
            .indexes
            .height
            .epoch
            .collect_one(current_height)
            .data()?;
        let current_epoch_usize: usize = current_epoch.into();

        let epoch_start_height = computer
            .indexes
            .epoch
            .first_height
            .collect_one(current_epoch)
            .data()?;
        let epoch_start_u32: u32 = epoch_start_height.into();

        let next_retarget_height = epoch_start_u32 + BLOCKS_PER_EPOCH;
        let blocks_into_epoch = current_height_u32 - epoch_start_u32;
        let remaining_blocks = next_retarget_height - current_height_u32;
        let progress_percent = (blocks_into_epoch as f64 / BLOCKS_PER_EPOCH as f64) * 100.0;

        let epoch_start_timestamp = computer
            .indexes
            .timestamp
            .epoch
            .collect_one(current_epoch)
            .data()?;
        let current_timestamp = indexer
            .vecs()
            .blocks
            .timestamp
            .collect_one(current_height)
            .data()?;

        // Bitcoin block timestamps can step backward within MTP rules, so
        // saturate the subtraction to avoid u32 underflow on a backwards-going
        // first block of an epoch.
        let elapsed_time = u64::from((*current_timestamp).saturating_sub(*epoch_start_timestamp));
        let time_avg = if blocks_into_epoch > 0 {
            elapsed_time / blocks_into_epoch as u64
        } else {
            TARGET_BLOCK_TIME
        };

        // Per-block time needed over remaining blocks to land the epoch at
        // BLOCKS_PER_EPOCH * TARGET_BLOCK_TIME (the convergence path that
        // client UIs render as adjustedTimeAvg).
        let target_total = BLOCKS_PER_EPOCH as u64 * TARGET_BLOCK_TIME;
        let adjusted_time_avg = if remaining_blocks > 0 {
            target_total.saturating_sub(elapsed_time) / remaining_blocks as u64
        } else {
            TARGET_BLOCK_TIME
        };

        let remaining_time = remaining_blocks as u64 * adjusted_time_avg;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(u64::from(*current_timestamp));
        let estimated_retarget_date = now + remaining_time;

        let expected_time = blocks_into_epoch as u64 * TARGET_BLOCK_TIME;
        let difficulty_change = if elapsed_time > 0 && blocks_into_epoch > 0 {
            ((expected_time as f64 / elapsed_time as f64) - 1.0) * 100.0
        } else {
            0.0
        };

        let time_offset = expected_time as i64 - elapsed_time as i64;

        let (previous_retarget, previous_time) = if current_epoch_usize > 0 {
            let prev_epoch = Epoch::from(current_epoch_usize - 1);
            let prev_epoch_start = computer
                .indexes
                .epoch
                .first_height
                .collect_one(prev_epoch)
                .data()?;

            let prev_difficulty = indexer
                .vecs()
                .blocks
                .difficulty
                .collect_one(prev_epoch_start)
                .data()?;
            let curr_difficulty = indexer
                .vecs()
                .blocks
                .difficulty
                .collect_one(epoch_start_height)
                .data()?;

            let retarget = if *prev_difficulty > 0.0 {
                ((*curr_difficulty / *prev_difficulty) - 1.0) * 100.0
            } else {
                0.0
            };

            (retarget, epoch_start_timestamp)
        } else {
            (0.0, epoch_start_timestamp)
        };

        let expected_blocks = elapsed_time as f64 / TARGET_BLOCK_TIME as f64;

        Ok(DifficultyAdjustment {
            progress_percent,
            difficulty_change,
            estimated_retarget_date: estimated_retarget_date * 1000,
            remaining_blocks,
            remaining_time: remaining_time * 1000,
            previous_retarget,
            previous_time,
            next_retarget_height: Height::from(next_retarget_height),
            time_avg: time_avg * 1000,
            adjusted_time_avg: adjusted_time_avg * 1000,
            time_offset,
            expected_blocks,
        })
    }
}
