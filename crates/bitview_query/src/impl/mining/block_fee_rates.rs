use bitview_types::{BlockFeeRatesEntry, FeeRatePercentiles, TimePeriod};

use brk_types::FeeRate;

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

        let read = |vec| -> Result<Vec<FeeRate>> {
            Ok(bw.read(vec)?.into_iter().map(mempool_fee_rate).collect())
        };
        let min = read(&frd.min.height)?;
        let pct10 = read(&frd.pct10.height)?;
        let pct25 = read(&frd.pct25.height)?;
        let median = read(&frd.median.height)?;
        let pct75 = read(&frd.pct75.height)?;
        let pct90 = read(&frd.pct90.height)?;
        let max = read(&frd.max.height)?;
        drop(pin);

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
