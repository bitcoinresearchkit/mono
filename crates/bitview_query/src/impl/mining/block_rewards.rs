use std::convert::identity;

use bitview_types::{BlockRewardsEntry, TimePeriod};

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
        let rewards =
            bw.means_rounded(&self.plugins().mining.rewards.coinbase.block.sats, identity)?;
        let prices = bw.mean_prices(&self.plugins().price.spot.cents.height)?;
        drop(pin);

        Ok(bw
            .buckets
            .iter()
            .zip(rewards.into_iter().zip(prices))
            .map(|(b, (sats, usd))| BlockRewardsEntry {
                avg_height: b.avg_height,
                timestamp: b.avg_timestamp,
                avg_rewards: sats,
                usd,
            })
            .collect())
    }
}
