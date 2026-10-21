use bitview_types::{BlockFeeRatesEntry, FeeRatePercentiles, TimePeriod};

use super::block_window::BlockWindow;
use crate::{Query, Result, r#impl::mempool_fee_rate};

impl Query {
    /// Mean effective (CPFP-adjusted) fee-rate percentiles per time bucket over `time_period`,
    /// chronological.
    pub fn block_fee_rates(&self, time_period: TimePeriod) -> Result<Vec<BlockFeeRatesEntry>> {
        let pin = self.pin_safe_lengths()?;
        let tip = Query::tip_height(&pin)?;
        let bw = BlockWindow::new(self, time_period, tip)?;
        let frd = &self
            .plugins()
            .transactions
            .fees
            .effective_fee_rate
            .distribution;

        let means = |vec| bw.means(vec, mempool_fee_rate);
        let min = means(&frd.min.height)?;
        let p10 = means(&frd.p10.height)?;
        let p25 = means(&frd.p25.height)?;
        let median = means(&frd.median.height)?;
        let p75 = means(&frd.p75.height)?;
        let p90 = means(&frd.p90.height)?;
        let max = means(&frd.max.height)?;
        drop(pin);

        Ok(bw
            .buckets
            .iter()
            .enumerate()
            .map(|(i, b)| BlockFeeRatesEntry {
                avg_height: b.avg_height,
                timestamp: b.avg_timestamp,
                percentiles: FeeRatePercentiles {
                    min: min[i],
                    p10: p10[i],
                    p25: p25[i],
                    median: median[i],
                    p75: p75[i],
                    p90: p90[i],
                    max: max[i],
                },
            })
            .collect())
    }
}
