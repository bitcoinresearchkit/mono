use bitview_primitives::Epoch;
use bitview_types::DifficultyAdjustmentEntry;
use brk_types::Height;
use vecdb::{ReadableVec, VecIndex};

use crate::{Error, Result, query_plugins::QueryPlugins};

/// One entry per retarget whose first block lies in `[start_height, end_height]`, with the
/// new/previous difficulty ratio (1.068 = +6.8%); `Error::Internal` for missing published data.
pub fn difficulty_epochs(
    plugins: &QueryPlugins,
    start_height: usize,
    end_height: usize,
) -> Result<Vec<DifficultyAdjustmentEntry>> {
    let start_epoch = Epoch::from(Height::from(start_height)).to_usize();
    let end_epoch = Epoch::from(Height::from(end_height)).to_usize();
    let mut height_cursor = plugins.mappings.epoch.first_height.cursor();
    let mut timestamp_cursor = plugins.mappings.timestamp.epoch.cursor();
    let mut difficulty_cursor = plugins.blocks.difficulty.value.epoch.cursor();

    let count = end_epoch
        .checked_sub(start_epoch)
        .ok_or(Error::Internal("Reversed mining epoch window"))?
        + 1;
    let mut results = Vec::with_capacity(count);
    let mut prev_difficulty: Option<f64> = None;

    // Include the predecessor even when the requested window starts exactly at
    // a retarget. Otherwise the first returned adjustment incorrectly becomes 0.
    for epoch_usize in start_epoch.saturating_sub(1)..=end_epoch {
        let epoch_height = height_cursor
            .get(epoch_usize)
            .ok_or(Error::Internal("Missing difficulty epoch start height"))?;

        // Epochs that start before the window are skipped; we still record
        // their difficulty so the next in-window entry can compute its ratio.
        if epoch_height.to_usize() < start_height {
            prev_difficulty = Some(
                *difficulty_cursor
                    .get(epoch_usize)
                    .ok_or(Error::Internal("Missing pre-window epoch difficulty"))?,
            );
            continue;
        }

        let epoch_timestamp = timestamp_cursor
            .get(epoch_usize)
            .ok_or(Error::Internal("Missing difficulty epoch timestamp"))?;
        let epoch_difficulty = *difficulty_cursor
            .get(epoch_usize)
            .ok_or(Error::Internal("Missing epoch difficulty"))?;

        let adjustment = match prev_difficulty {
            Some(prev) => epoch_difficulty / prev,
            None => 0.0,
        };

        results.push(DifficultyAdjustmentEntry {
            timestamp: epoch_timestamp,
            height: epoch_height,
            difficulty: epoch_difficulty,
            adjustment,
        });

        prev_difficulty = Some(epoch_difficulty);
    }

    Ok(results)
}
