use std::ops::Sub;

use brk_error::{Error, OptionData, Result};
use brk_types::{Height, RewardStats};
use vecdb::{AnyVec, ReadableVec, VecIndex, VecValue};

use crate::Query;

impl Query {
    /// Reads coinbase rewards, fees, and tx counts over the last `block_count`
    /// blocks from their cumulative sources. Errors `OutOfRange` if
    /// `block_count` is zero, and `Internal` if any source is stamped short of
    /// the tip.
    pub fn reward_stats(&self, block_count: usize) -> Result<RewardStats> {
        if block_count == 0 {
            return Err(Error::OutOfRange("block_count must be >= 1".into()));
        }

        let plugins = self.plugins();
        let current_height = self.height();

        let end_block = current_height;
        let start_block = Height::from(current_height.to_usize().saturating_sub(block_count - 1));

        let coinbase_vec = &plugins.mining.rewards.coinbase.cumulative.sats.height;
        let fee_vec = &plugins.mining.rewards.fees.cumulative.sats.height;
        let tx_count_vec = &plugins.transactions.count.total.cumulative.height;

        let end = end_block.to_usize() + 1;

        if coinbase_vec.len() < end || fee_vec.len() < end || tx_count_vec.len() < end {
            return Err(Error::Internal(
                "reward stats vecs lag the tip; retry once indexing catches up",
            ));
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
    T: Sub<Output = T> + VecValue,
{
    let start = start.to_usize();
    let end = end.to_usize();

    if start == 0 {
        return cumulative.collect_one_at(end).data();
    }

    if end - start < cumulative.cursor_chunk_size() {
        let mut previous = None;
        let end_value = cumulative
            .fold_range_at(start - 1, end + 1, None, |_, value| {
                previous.get_or_insert_with(|| value.clone());
                Some(value)
            })
            .data()?;

        return Ok(end_value - previous.data()?);
    }

    let end_value = cumulative.collect_one_at(end).data()?;
    let previous = cumulative.collect_one_at(start - 1).data()?;
    Ok(end_value - previous)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use brk_types::{Sats, Version};
    use vecdb::{
        AnyStoredVec, Database, EagerVec, ImportableVec, PcoVec, ReadOnlyClone, ReadableVec,
        WritableVec,
    };

    use super::*;

    #[test]
    fn cumulative_delta_matches_range_sum_across_read_strategies() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "bitview-reward-stats-{}-{suffix}",
            std::process::id()
        ));
        let db = Database::open(&path).unwrap();
        let mut cumulative: EagerVec<PcoVec<Height, Sats>> =
            EagerVec::forced_import(&db, "cumulative", Version::ONE).unwrap();
        let values = (0_usize..2_051)
            .map(|index| Sats::from(index + 1))
            .collect::<Vec<_>>();
        let mut total = Sats::ZERO;

        for value in &values {
            total += *value;
            cumulative.push(total);
        }
        cumulative.write().unwrap();

        let read_only = cumulative.read_only_clone();
        let page = read_only.cursor_chunk_size();
        for (start, end) in [
            (0, 0),
            (0, values.len() - 1),
            (values.len() - 1, values.len() - 1),
            (values.len() - page, values.len() - 1),
            (values.len() - page - 1, values.len() - 1),
            (32, 1_750),
        ] {
            let expected = values[start..=end].iter().copied().sum();
            let actual =
                cumulative_delta(&read_only, Height::from(start), Height::from(end)).unwrap();
            assert_eq!(actual, expected, "range {start}..={end}");
        }

        drop(read_only);
        drop(cumulative);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}
