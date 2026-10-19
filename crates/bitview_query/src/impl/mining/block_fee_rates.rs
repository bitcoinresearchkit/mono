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
                percentiles: FeeRatePercentiles {
                    min: min[i],
                    pct10: pct10[i],
                    pct25: pct25[i],
                    median: median[i],
                    pct75: pct75[i],
                    pct90: pct90[i],
                    max: max[i],
                },
            })
            .collect())
    }
}
