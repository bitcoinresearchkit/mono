use brk_types::{BlockRewardsEntry, Cents, Dollars, Sats, TimePeriod};

use super::block_window::BlockWindow;
use crate::Query;

impl Query {
    /// Time-bucketed average block rewards (subsidy + fees) over
    /// `time_period`. One entry per bucket, ordered chronologically. Each
    /// entry carries the bucket's average height/timestamp, the round-half-up
    /// mean of coinbase rewards in sats, and the bucket-mean USD spot price
    /// (the spot price, not rewards-in-USD: clients multiply).
    pub fn block_rewards(
        &self,
        time_period: TimePeriod,
    ) -> brk_error::Result<Vec<BlockRewardsEntry>> {
        let bw = BlockWindow::new(self, time_period)?;
        let rewards: Vec<Sats> = bw.read(&self.computer().mining.rewards.coinbase.block.sats)?;
        let prices: Vec<Cents> = bw.read(&self.computer().price.spot.cents.height)?;

        Ok(bw
            .buckets
            .iter()
            .map(|b| BlockRewardsEntry {
                avg_height: b.avg_height,
                timestamp: b.avg_timestamp,
                avg_rewards: b.mean_rounded(&rewards),
                usd: Dollars::from(b.mean_rounded(&prices)),
            })
            .collect())
    }
}
