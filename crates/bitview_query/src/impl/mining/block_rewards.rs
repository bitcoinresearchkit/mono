use bitview_types::{BlockRewardsEntry, TimePeriod};
use brk_types::{Cents, Sats};

use super::block_window::BlockWindow;
use crate::{Query, Result};

impl Query {
    /// Time-bucketed average block rewards (subsidy + fees) over
    /// `time_period`. One entry per bucket, ordered chronologically. Each
    /// entry carries the bucket's average height/timestamp, the round-half-up
    /// mean of coinbase rewards in sats, and the bucket-mean USD spot price
    /// (the spot price, not rewards-in-USD: clients multiply).
    pub fn block_rewards(&self, time_period: TimePeriod) -> Result<Vec<BlockRewardsEntry>> {
        let pin = self.pin_safe_lengths()?;
        let tip = Query::tip_height(&pin);
        let bw = BlockWindow::new(self, time_period, tip)?;
        let rewards: Vec<Sats> = bw.read(&self.plugins().mining.rewards.coinbase.block.sats)?;
        let prices: Vec<Cents> = bw.read(&self.plugins().price.spot.cents.height)?;
        drop(pin);

        Ok(bw
            .buckets
            .iter()
            .map(|b| BlockRewardsEntry {
                avg_height: b.avg_height,
                timestamp: b.avg_timestamp,
                avg_rewards: b.mean_rounded(&rewards),
                usd: b.mean_price(&prices),
            })
            .collect())
    }
}
