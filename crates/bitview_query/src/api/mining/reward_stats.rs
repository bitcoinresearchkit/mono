use bitview_types::RewardStats;
use brk_types::Height;
use vecdb::{AnyVec, VecIndex};

use super::cumulative_delta;
use crate::{Error, Query, Result, TipPin};

impl Query {
    /// Coinbase rewards, fees and transaction counts over the last `block_count` blocks (fewer
    /// near genesis). `InvalidParam` for zero, `Internal` for incomplete published data.
    pub fn reward_stats(&self, at: &TipPin, block_count: usize) -> Result<RewardStats> {
        if block_count == 0 {
            return Err(Error::InvalidParam("block_count must be at least 1".into()));
        }

        let end_block = at.height;
        let start_block = Height::from(end_block.to_usize().saturating_sub(block_count - 1));
        let plugins = self.plugins();

        let coinbase_vec = &plugins.mining.rewards.coinbase.cumulative.sats.height;
        let fee_vec = &plugins.mining.rewards.fees.cumulative.sats.height;
        let tx_count_vec = &plugins.transactions.count.cumulative.height;

        let end = end_block.to_usize() + 1;

        if coinbase_vec.len() < end || fee_vec.len() < end || tx_count_vec.len() < end {
            return Err(Error::Internal("Incomplete published reward data"));
        }

        let total_reward = cumulative_delta(coinbase_vec, start_block, end_block)?;
        let total_fee = cumulative_delta(fee_vec, start_block, end_block)?;
        let total_tx = cumulative_delta(tx_count_vec, start_block, end_block)?.into();

        Ok(RewardStats {
            start_block,
            end_block,
            total_reward,
            total_fee,
            total_tx,
        })
    }
}
