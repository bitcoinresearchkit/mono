use brk_types::{BlockFeesEntry, Cents, Dollars, Sats, TimePeriod};

use super::block_window::BlockWindow;
use crate::Query;

impl Query {
    /// Time-bucketed average block fees over `time_period`. One entry per
    /// bucket, ordered chronologically. Each entry carries the bucket's
    /// average height/timestamp, the round-half-up mean of block fees in
    /// sats, and the bucket-mean USD spot price (the spot price, not
    /// fees-in-USD: clients multiply).
    pub fn block_fees(&self, time_period: TimePeriod) -> brk_error::Result<Vec<BlockFeesEntry>> {
        let bw = BlockWindow::new(self, time_period)?;
        let fees: Vec<Sats> = bw.read(&self.plugins().mining.rewards.fees.block.sats)?;
        let prices: Vec<Cents> = bw.read(&self.plugins().price.spot.cents.height)?;

        Ok(bw
            .buckets
            .iter()
            .map(|b| BlockFeesEntry {
                avg_height: b.avg_height,
                timestamp: b.avg_timestamp,
                avg_fees: b.mean_rounded(&fees),
                usd: Dollars::from(b.mean_rounded(&prices)),
            })
            .collect())
    }
}
