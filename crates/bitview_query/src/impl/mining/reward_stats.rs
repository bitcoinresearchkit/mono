use bitview_types::RewardStats;
use brk_types::Height;
use vecdb::{AnyVec, CheckedSub, ReadableVec, VecIndex, VecValue};

use crate::{Error, OptionData, Query, Result};

impl Query {
    /// Coinbase rewards, fees and transaction counts over the last `block_count` blocks (fewer
    /// near genesis). `InvalidParam` for zero, `Internal` for incomplete published data.
    pub fn reward_stats(&self, block_count: usize) -> Result<RewardStats> {
        if block_count == 0 {
            return Err(Error::InvalidParam("block_count must be >= 1".into()));
        }

        let pin = self.pin_safe_lengths()?;
        let end_block = Query::tip_height(&pin)?;
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

fn cumulative_delta<T>(
    cumulative: &impl ReadableVec<Height, T>,
    start: Height,
    end: Height,
) -> Result<T>
where
    T: CheckedSub + VecValue,
{
    let start = start.to_usize();
    let end = end.to_usize();

    if start == 0 {
        return cumulative.collect_one_at(end).data();
    }

    debug_assert!(start <= end);
    let distance = end - start;
    if distance < cumulative.cursor_chunk_size() {
        let mut previous = None;
        let end_value = cumulative
            .fold_range_at(start - 1, end + 1, None, |_, value| {
                previous.get_or_insert_with(|| value.clone());
                Some(value)
            })
            .data()?;

        return end_value
            .checked_sub(previous.data()?)
            .ok_or(Error::Internal("Decreasing cumulative rewards"));
    }

    let end_value = cumulative.collect_one_at(end).data()?;
    let previous = cumulative.collect_one_at(start - 1).data()?;
    end_value
        .checked_sub(previous)
        .ok_or(Error::Internal("Decreasing cumulative rewards"))
}
