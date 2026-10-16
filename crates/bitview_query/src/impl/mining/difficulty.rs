use std::time::{SystemTime, UNIX_EPOCH};

use bitview_primitives::Epoch;
use bitview_types::DifficultyAdjustment;
use brk_types::{BLOCKS_PER_DIFF_EPOCHS as BLOCKS_PER_EPOCH, Height};
use vecdb::ReadableVec;

use crate::{OptionData, Query, Result};

/// Target block time in seconds (10 minutes)
const TARGET_BLOCK_TIME: u64 = 600;
/// The early-epoch projection window, in blocks (mempool.space's quarter epoch).
const QUARTER_EPOCH: u32 = 503;

impl Query {
    /// Live difficulty-adjustment snapshot for the current epoch, as mempool.space computes it
    /// on mainnet but measured to the tip block's time instead of the wall clock (cacheable per
    /// block). Bundles progress through the 2016-block window, the projection pace
    /// (`adjustedTimeAvg`) with the next-retarget percentage, remaining blocks and time at it, an estimated wall-clock retarget time, and the previous
    /// retarget percentage (current vs previous epoch first-block difficulty). Output time
    /// fields are in milliseconds.
    pub fn difficulty_adjustment(&self) -> Result<DifficultyAdjustment> {
        let pin = self.pin_safe_lengths()?;
        let indexer = self.indexer();
        let plugins = self.plugins();
        let current_height = Query::tip_height(&pin)?;
        let current_height_u32: u32 = current_height.into();

        let current_epoch = Epoch::from(current_height);
        let blocks_into_epoch = current_height_u32 % BLOCKS_PER_EPOCH;
        let epoch_start_u32 = current_height_u32 - blocks_into_epoch;
        let epoch_start_height = Height::from(epoch_start_u32);

        let next_retarget_height = epoch_start_u32 + BLOCKS_PER_EPOCH;
        let remaining_blocks = next_retarget_height - current_height_u32;
        let progress_percent = (blocks_into_epoch as f64 / BLOCKS_PER_EPOCH as f64) * 100.0;

        let epoch_start_timestamp = plugins
            .mappings
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
        // Observed pace; the target pace until the epoch shows some elapsed time (a
        // backward-stepping timestamp leaves none).
        let observed = blocks_into_epoch > 0 && elapsed_time > 0;
        let time_avg_ms = if observed {
            elapsed_time * 1000 / blocks_into_epoch as u64
        } else {
            TARGET_BLOCK_TIME * 1000
        };

        let previous_retarget = if epoch_start_u32 > 0 {
            let prev_epoch_start = Height::from(epoch_start_u32 - BLOCKS_PER_EPOCH);
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

            if *prev_difficulty > 0.0 {
                ((*curr_difficulty / *prev_difficulty) - 1.0) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        // mempool.space's projection: early in an epoch, the last quarter epoch's blocks, those
        // before the retarget scaled by it; otherwise the observed pace. As a span (ms) over the
        // block intervals it covers: multiplied before dividing, a pace under a millisecond per
        // block still projects time. A span of no time (backward-stepping timestamps) projects
        // nothing: the target pace, no change.
        let quarter_span_ms = if epoch_start_u32 > 0 && blocks_into_epoch < QUARTER_EPOCH {
            let quarter_start = indexer
                .vecs()
                .blocks
                .timestamp
                .collect_one(Height::from(current_height_u32 - QUARTER_EPOCH))
                .data()?;
            let before_retarget = (*epoch_start_timestamp).saturating_sub(*quarter_start);
            let scaled = f64::from(before_retarget) * (1.0 + previous_retarget / 100.0);
            ((elapsed_time as f64 + scaled) * 1000.0) as u64
        } else {
            0
        };
        let projection = if quarter_span_ms > 0 {
            Some((quarter_span_ms, u64::from(QUARTER_EPOCH)))
        } else if observed {
            Some((elapsed_time * 1000, u64::from(blocks_into_epoch)))
        } else {
            None
        };
        let (span_ms, span_blocks) = projection.unwrap_or((TARGET_BLOCK_TIME * 1000, 1));
        let adjusted_time_avg_ms = span_ms / span_blocks;
        let remaining_time_ms = span_ms * u64::from(remaining_blocks) / span_blocks;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(u64::from(*current_timestamp));
        let estimated_retarget_date_ms = now * 1000 + remaining_time_ms;

        // The retarget at the projected pace, as Bitcoin's rule counts it: one block more than
        // the intervals its span covers (exact at an epoch's last block). A retarget changes
        // difficulty by at most a factor of four either way.
        let difficulty_change = projection.map_or(0.0, |(span_ms, span_blocks)| {
            let target_ms = TARGET_BLOCK_TIME * 1000 * (span_blocks + 1);
            ((target_ms as f64 / span_ms as f64 - 1.0) * 100.0).clamp(-75.0, 300.0)
        });

        let expected_blocks = elapsed_time as f64 / TARGET_BLOCK_TIME as f64;

        Ok(DifficultyAdjustment {
            progress_percent,
            difficulty_change,
            estimated_retarget_date: estimated_retarget_date_ms,
            remaining_blocks,
            remaining_time: remaining_time_ms,
            previous_retarget,
            previous_time: epoch_start_timestamp,
            next_retarget_height: Height::from(next_retarget_height),
            time_avg: time_avg_ms,
            adjusted_time_avg: adjusted_time_avg_ms,
            // mempool.space offsets testnet's 20-minute rule only.
            time_offset: 0,
            expected_blocks,
        })
    }
}
