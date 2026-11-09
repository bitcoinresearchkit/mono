use bitview_compute::ComputeRollingStats;
use bitview_plugin_mappings::LookbackVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::Lengths;
use bitview_vecs::priced_starts;
use brk_error::Result;
use brk_exit::Exit;
use vecdb::ComputeVec;

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        lookback: &LookbackVecs,
        prices: &PriceVecs,
        starting_lengths: Lengths,
        exit: &Exit,
    ) -> Result<()> {
        let close = &prices.spot.cents.height;

        self.sma_prefix_sum
            .compute_cumulative(starting_lengths.height, close, exit)?;

        for (days, state) in self.ema_stored.iter_mut() {
            state.compute_rolling_ema(
                starting_lengths.height,
                &priced_starts(lookback.start_vec(days)),
                close,
                exit,
            )?;
        }

        let from = starting_lengths.height;
        self.sma.compute_ratios(from, close, exit)?;
        for (_, ema) in self.ema.iter_mut() {
            ema.compute_ratio(from, close, exit)?;
        }

        Ok(())
    }
}
