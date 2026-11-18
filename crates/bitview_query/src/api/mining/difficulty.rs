use bitview_primitives::DifficultyEpoch;
use bitview_types::DifficultyAdjustment;
use brk_types::Height;
use vecdb::ReadableVec;

use crate::{OptionData, Query, Result, TipPin};

/// Target block time in seconds (10 minutes)
const TARGET_BLOCK_TIME: u64 = 600;

impl Query {
    /// The current epoch's difficulty-adjustment snapshot, as mempool.space computes it on
    /// mainnet but with elapsed time measured to the tip block (cacheable per block).
    /// `previous_retarget` compares the current and previous epochs' first-block difficulties.
    /// Time fields are in milliseconds. The estimated retarget date counts from the tip block's
    /// time, not the clock (where mempool.space adds to its own): the answer is a function of
    /// the chain, the same on every server until the next block, and `remaining_time` is there
    /// for a reader counting from now.
    pub fn difficulty_adjustment(&self, at: &TipPin) -> Result<DifficultyAdjustment> {
        let interval = brk_types::chain().difficulty_interval;
        let quarter_epoch = interval / 4 - 1;
        let indexer = self.indexer();
        let plugins = self.plugins();
        let current_height = at.height;
        let current_height_u32: u32 = current_height.into();

        let current_epoch = DifficultyEpoch::from(current_height);
        let blocks_into_epoch = current_height_u32 % interval;
        let epoch_start_u32 = current_height_u32 - blocks_into_epoch;
        let epoch_start_height = Height::from(epoch_start_u32);

        let next_retarget_height = epoch_start_u32 + interval;
        let remaining_blocks = next_retarget_height - current_height_u32;
        let progress_percent = (blocks_into_epoch as f64 / interval as f64) * 100.0;

        let epoch_start_timestamp = plugins
            .mappings
            .timestamp
            .difficulty_epoch
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
            let prev_epoch_start = Height::from(epoch_start_u32 - interval);
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

            ((*curr_difficulty / *prev_difficulty) - 1.0) * 100.0
        } else {
            0.0
        };

        // mempool.space's projection: early in an epoch, the last quarter epoch's blocks, those
        // before the retarget scaled by it; otherwise the observed pace. As a span (ms) over the
        // block intervals it covers: multiplied before dividing, a pace under a millisecond per
        // block still projects time. A span of no time (backward-stepping timestamps) projects
        // nothing: the target pace, no change.
        let quarter_span_ms = if epoch_start_u32 > 0 && blocks_into_epoch < quarter_epoch {
            let quarter_start = indexer
                .vecs()
                .blocks
                .timestamp
                .collect_one(Height::from(current_height_u32 - quarter_epoch))
                .data()?;
            let before_retarget = (*epoch_start_timestamp).saturating_sub(*quarter_start);
            let scaled = f64::from(before_retarget) * (1.0 + previous_retarget / 100.0);
            ((elapsed_time as f64 + scaled) * 1000.0) as u64
        } else {
            0
        };
        let projection = if quarter_span_ms > 0 {
            Some((quarter_span_ms, u64::from(quarter_epoch)))
        } else if observed {
            Some((elapsed_time * 1000, u64::from(blocks_into_epoch)))
        } else {
            None
        };
        let (span_ms, span_blocks) = projection.unwrap_or((TARGET_BLOCK_TIME * 1000, 1));
        let adjusted_time_avg_ms = span_ms / span_blocks;
        let remaining_time_ms = span_ms * u64::from(remaining_blocks) / span_blocks;
        let estimated_retarget_date_ms = u64::from(*current_timestamp) * 1000 + remaining_time_ms;

        // The retarget at the projected pace, as Bitcoin's rule counts it: one block more than
        // the intervals its span covers (exact at an epoch's last block). A retarget changes
        // difficulty by at most a factor of four either way.
        let difficulty_change = projection
            .filter(|_| {
                !bitcoin::params::Params::new(brk_types::chain().network).no_pow_retargeting
            })
            .map_or(0.0, |(span_ms, span_blocks)| {
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
