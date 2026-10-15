use bitview_primitives::Epoch;
use bitview_types::DifficultyAdjustmentEntry;
use brk_types::Height;
use vecdb::{ReadableVec, VecIndex};

use crate::{Error, Result, query_plugins::QueryPlugins};

/// Walk every difficulty epoch overlapping `[start_height, end_height]` and
/// return one `DifficultyAdjustmentEntry` per retarget whose first block
/// lies inside the window. Each entry carries the epoch's first-block
/// timestamp/height, the epoch's difficulty, and the new/previous difficulty
/// ratio (e.g. 1.068 = +6.8%, matching the field's contract). Epochs whose
/// first block falls before `start_height` are skipped but their difficulty
/// is still read so the next in-window entry can compute its ratio. Returns
/// `Error::Internal` on any missing cursor read so corrupt zero-valued
/// entries cannot slip into the output under per-vec stamp lag.
pub fn iter_difficulty_epochs(
    plugins: &QueryPlugins,
    start_height: usize,
    end_height: usize,
) -> Result<Vec<DifficultyAdjustmentEntry>> {
    let start_epoch = Epoch::from(Height::from(start_height)).to_usize();
    let end_epoch = Epoch::from(Height::from(end_height)).to_usize();
    let mut height_cursor = plugins.mappings.epoch.first_height.cursor();
    let mut timestamp_cursor = plugins.mappings.timestamp.epoch.cursor();
    let mut difficulty_cursor = plugins.blocks.difficulty.block.epoch.cursor();

    let count = end_epoch
        .checked_sub(start_epoch)
        .ok_or(Error::Internal("Reversed mining epoch window"))?
        + 1;
    let mut results = Vec::with_capacity(count);
    let mut prev_difficulty: Option<f64> = None;

    // Include the predecessor even when the requested window starts exactly at
    // a retarget. Otherwise the first returned adjustment incorrectly becomes 0.
    for epoch_usize in start_epoch.saturating_sub(1)..=end_epoch {
        let epoch_height = height_cursor.get(epoch_usize).ok_or(Error::Internal(
            "iter_difficulty_epochs: missing epoch first_height",
        ))?;

        // Epochs that start before the window are skipped; we still record
        // their difficulty so the next in-window entry can compute its ratio.
        if epoch_height.to_usize() < start_height {
            prev_difficulty = Some(*difficulty_cursor.get(epoch_usize).ok_or(Error::Internal(
                "iter_difficulty_epochs: missing pre-window epoch difficulty",
            ))?);
            continue;
        }

        let epoch_timestamp = timestamp_cursor.get(epoch_usize).ok_or(Error::Internal(
            "iter_difficulty_epochs: missing epoch timestamp",
        ))?;
        let epoch_difficulty = *difficulty_cursor.get(epoch_usize).ok_or(Error::Internal(
            "iter_difficulty_epochs: missing epoch difficulty",
        ))?;

        let change_percent = match prev_difficulty {
            Some(prev) if prev > 0.0 => epoch_difficulty / prev,
            _ => 0.0,
        };

        results.push(DifficultyAdjustmentEntry {
            timestamp: epoch_timestamp,
            height: epoch_height,
            difficulty: epoch_difficulty,
            change_percent,
        });

        prev_difficulty = Some(epoch_difficulty);
    }

    Ok(results)
}
