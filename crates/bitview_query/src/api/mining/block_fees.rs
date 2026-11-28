use bitview_types::{BlockFeesEntry, TimePeriod};

use super::block_window::BlockWindow;
use crate::{Query, Result, TipPin};

impl Query {
    /// Round-half-up mean block fees in sats per time bucket over `time_period`, chronological,
    /// with the bucket's mean USD spot price (not fees in USD: clients multiply).
    pub fn block_fees(&self, at: &TipPin, time_period: TimePeriod) -> Result<Vec<BlockFeesEntry>> {
        let tip = at.height;
        let bw = BlockWindow::new(self, time_period, tip)?;
        let fees = bw.means_from_totals(&self.plugins().mining.rewards.fees.cumulative.sats.height)?;
        let prices = bw.mean_prices(&self.plugins().price.spot.cents.height)?;

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
