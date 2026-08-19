use brk_types::{BlockFeeRatesEntry, FeeRatePercentiles, TimePeriod};

use super::block_window::BlockWindow;
use crate::Query;

impl Query {
    /// Time-bucketed fee-rate percentiles over `time_period`. One entry per
    /// bucket, ordered chronologically. Each entry carries the bucket's
    /// average height/timestamp and the seven percentile means
    /// (`min, pct10, pct25, median, pct75, pct90, max`).
    pub fn block_fee_rates(
        &self,
        time_period: TimePeriod,
    ) -> brk_error::Result<Vec<BlockFeeRatesEntry>> {
        let bw = BlockWindow::new(self, time_period)?;
        let frd = &self
            .computer()
            .transactions
            .fees
            .effective_fee_rate
            .distribution
            .block;

        let min = bw.read(&frd.min.height)?;
        let pct10 = bw.read(&frd.pct10.height)?;
        let pct25 = bw.read(&frd.pct25.height)?;
        let median = bw.read(&frd.median.height)?;
        let pct75 = bw.read(&frd.pct75.height)?;
        let pct90 = bw.read(&frd.pct90.height)?;
        let max = bw.read(&frd.max.height)?;

        Ok(bw
            .buckets
            .iter()
            .map(|b| BlockFeeRatesEntry {
                avg_height: b.avg_height,
                timestamp: b.avg_timestamp,
                percentiles: FeeRatePercentiles::new(
                    b.mean(&min),
                    b.mean(&pct10),
                    b.mean(&pct25),
                    b.mean(&median),
                    b.mean(&pct75),
                    b.mean(&pct90),
                    b.mean(&max),
                ),
            })
            .collect())
    }
}
