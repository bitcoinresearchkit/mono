use brk_error::Error;
use brk_types::{Height, RewardStats, Sats};
use vecdb::{AnyVec, ReadableVec, VecIndex};

use crate::Query;

impl Query {
    /// Sums coinbase rewards, fees, and tx counts over the last `block_count`
    /// blocks ending at the current tip. Errors `OutOfRange` if `block_count`
    /// is zero, and `Internal` if any of the three per-block vecs (coinbase,
    /// fees, tx count) is stamped short of the tip - silent truncation by
    /// `fold_range_at` would otherwise produce a quietly low total.
    pub fn reward_stats(&self, block_count: usize) -> brk_error::Result<RewardStats> {
        if block_count == 0 {
            return Err(Error::OutOfRange("block_count must be >= 1".into()));
        }

        let plugins = self.plugins();
        let current_height = self.height();

        let end_block = current_height;
        let start_block = Height::from(current_height.to_usize().saturating_sub(block_count - 1));

        let coinbase_vec = &plugins.mining.rewards.coinbase.block.sats;
        let fee_vec = &plugins.mining.rewards.fees.block.sats;
        let tx_count_vec = &plugins.transactions.count.total.block;

        let start = start_block.to_usize();
        let end = end_block.to_usize() + 1;

        if coinbase_vec.len() < end || fee_vec.len() < end || tx_count_vec.len() < end {
            return Err(Error::Internal(
                "reward stats vecs lag the tip; retry once indexing catches up",
            ));
        }

        let total_reward = coinbase_vec.fold_range_at(start, end, Sats::ZERO, |acc, v| acc + v);
        let total_fee = fee_vec.fold_range_at(start, end, Sats::ZERO, |acc, v| acc + v);
        let total_tx = tx_count_vec.fold_range_at(start, end, 0u64, |acc, v| acc + *v);

        Ok(RewardStats {
            start_block,
            end_block,
            total_reward,
            total_fee,
            total_tx,
        })
    }
}
