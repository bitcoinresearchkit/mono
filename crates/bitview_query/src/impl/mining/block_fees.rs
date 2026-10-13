use bitview_types::{BlockFeesEntry, TimePeriod};
use brk_types::{Cents, Sats};

use super::block_window::BlockWindow;
use crate::{Query, Result};

impl Query {
    /// Time-bucketed average block fees over `time_period`. One entry per
    /// bucket, ordered chronologically. Each entry carries the bucket's
    /// average height/timestamp, the round-half-up mean of block fees in
    /// sats, and the bucket-mean USD spot price (the spot price, not
    /// fees-in-USD: clients multiply).
    pub fn block_fees(&self, time_period: TimePeriod) -> Result<Vec<BlockFeesEntry>> {
        let pin = self.pin_safe_lengths()?;
        let tip = Query::tip_height(&pin);
        let bw = BlockWindow::new(self, time_period, tip)?;
        let fees: Vec<Sats> = bw.read(&self.plugins().mining.rewards.fees.block.sats)?;
        let prices: Vec<Cents> = bw.read(&self.plugins().price.spot.cents.height)?;
        drop(pin);

        Ok(bw
            .buckets
            .iter()
            .map(|b| BlockFeesEntry {
                avg_height: b.avg_height,
                timestamp: b.avg_timestamp,
                avg_fees: b.mean_rounded(&fees),
                usd: b.mean_price(&prices),
            })
            .collect())
    }
}
