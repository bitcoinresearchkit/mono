use std::convert::identity;

use bitview_types::{BlockFeesEntry, TimePeriod};

use super::block_window::BlockWindow;
use crate::{Query, Result};

impl Query {
    /// Round-half-up mean block fees in sats per time bucket over `time_period`, chronological,
    /// with the bucket's mean USD spot price (not fees in USD: clients multiply).
    pub fn block_fees(&self, time_period: TimePeriod) -> Result<Vec<BlockFeesEntry>> {
        let pin = self.pin_safe_lengths()?;
        let tip = Query::tip_height(&pin)?;
        let bw = BlockWindow::new(self, time_period, tip)?;
        let fees = bw.means_rounded(&self.plugins().mining.rewards.fees.block.sats, identity)?;
        let prices = bw.mean_prices(&self.plugins().price.spot.cents.height)?;
        drop(pin);

        Ok(bw
            .buckets
            .iter()
            .zip(fees.into_iter().zip(prices))
            .map(|(b, (sats, usd))| BlockFeesEntry {
                avg_height: b.avg_height,
                timestamp: b.avg_timestamp,
                avg_fees: sats,
                usd,
            })
            .collect())
    }
}
