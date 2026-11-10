use std::convert::identity;

use bitview_types::{BlockFeesEntry, TimePeriod, TimestampRange};

use super::block_window::BlockWindow;
use crate::{Query, Result};

impl Query {
    /// Round-half-up mean block fees in sats per time bucket over `time_period`, chronological,
    /// with the bucket's mean on-chain USD spot price (not fees in USD).
    pub fn block_fees(&self, time_period: TimePeriod) -> Result<Vec<BlockFeesEntry>> {
        let pin = self.pin_safe_lengths()?;
        let bw = BlockWindow::new(self, time_period, Query::tip_height(&pin)?)?;
        self.block_fees_window(bw)
    }

    /// Inclusive raw header timestamps. The divisor uses the original requested
    /// span. One pin covers conservative bounds, exact filtering, fees and prices.
    /// USD keeps the named-period on-chain source/mean policy, not SQL's ungrouped price.
    pub fn block_fees_range(&self, range: TimestampRange) -> Result<Vec<BlockFeesEntry>> {
        let pin = self.pin_safe_lengths()?;
        let bw = BlockWindow::between(self, range, Query::tip_height(&pin)?)?;
        self.block_fees_window(bw)
    }

    fn block_fees_window(&self, bw: BlockWindow) -> Result<Vec<BlockFeesEntry>> {
        let fees = bw.means_rounded(&self.plugins().mining.rewards.fees.block.sats, identity)?;
        let prices = bw.mean_prices(&self.plugins().price.spot.cents.height)?;
        Ok(bw.buckets.iter().zip(fees.into_iter().zip(prices))
            .map(|(bucket, (sats, usd))| BlockFeesEntry {
                avg_height: bucket.avg_height,
                timestamp: bucket.avg_timestamp,
                avg_fees: sats,
                usd,
            })
            .collect())
    }
}
