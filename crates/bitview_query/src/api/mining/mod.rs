pub mod block_bucket;
pub mod block_fee_rates;
pub mod block_fees;
pub mod block_rewards;
pub mod block_sizes;
pub mod block_window;
pub mod difficulty;
pub mod difficulty_adjustments;
pub mod epochs;
pub mod hashrate;
pub mod period_start;
pub mod pool_blocks;
pub mod pools;
pub mod reward_stats;

pub use period_start::start_height;
pub use pool_blocks::ResolvedPoolBlocks;

use brk_types::Height;
use vecdb::{CheckedSub, ReadableVec, VecIndex, VecValue};

use crate::{Error, OptionData, Result};

/// `cumulative[end] - cumulative[start - 1]`: what accrued over blocks `start..=end` (from genesis
/// at `start == 0`). Two point reads, no allocation: `Internal` if one is missing or the column
/// decreases.
pub(crate) fn cumulative_delta<T>(
    cumulative: &impl ReadableVec<Height, T>,
    start: Height,
    end: Height,
) -> Result<T>
where
    T: CheckedSub + VecValue,
{
    debug_assert!(start <= end);
    let (start, end) = (start.to_usize(), end.to_usize());
    if start == 0 {
        return cumulative.collect_one_at(end).data();
    }
    let total = cumulative.collect_one_at(end).data()?;
    let prior = cumulative.collect_one_at(start - 1).data()?;
    total.checked_sub(prior).ok_or(Error::Internal("Decreasing cumulative column"))
}
