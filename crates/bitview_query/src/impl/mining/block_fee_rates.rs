use bitview_types::{BlockFeeRatesEntry, FeeRatePercentiles, TimePeriod};

use super::block_window::BlockWindow;
use crate::{Query, Result, r#impl::mempool_fee_rate};

impl Query {
    /// Time-bucketed fee-rate percentiles over `time_period`. One entry per
    /// bucket, ordered chronologically. Each entry carries the bucket's
    /// average height/timestamp and the seven percentile means
    /// (`min, pct10, pct25, median, pct75, pct90, max`).
    pub fn block_fee_rates(&self, time_period: TimePeriod) -> Result<Vec<BlockFeeRatesEntry>> {
        let pin = self.pin_safe_lengths()?;
        let tip = Query::tip_height(&pin);
        let bw = BlockWindow::new(self, time_period, tip)?;
        let frd = &self
            .plugins()
            .transactions
            .fees
            .effective_fee_rate
            .distribution;

        let means = |vec| bw.means(vec, mempool_fee_rate);
        let min = means(&frd.min.height)?;
        let pct10 = means(&frd.pct10.height)?;
        let pct25 = means(&frd.pct25.height)?;
        let median = means(&frd.median.height)?;
        let pct75 = means(&frd.pct75.height)?;
        let pct90 = means(&frd.pct90.height)?;
        let max = means(&frd.max.height)?;
        drop(pin);

        Ok(bw
            .buckets
            .iter()
            .enumerate()
            .map(|(i, b)| BlockFeeRatesEntry {
                avg_height: b.avg_height,
                timestamp: b.avg_timestamp,
                percentiles: FeeRatePercentiles::new(
                    min[i], pct10[i], pct25[i], median[i], pct75[i], pct90[i], max[i],
                ),
            })
            .collect())
    }
}
