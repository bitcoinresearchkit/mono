use bitview_compute::ComputeRollingStats;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::LookbackVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_vecs::priced_starts;
use brk_error::Result;
use brk_exit::Exit;

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        lookback: &LookbackVecs,
        prices: &PriceVecs,
        exit: &Exit,
    ) -> Result<()> {
        let from = indexer.compute_start().height;
        let close = &prices.spot.cents.height;

        for (_, sma) in self.sma.periods.iter_mut() {
            sma.source.compute(from, lookback, close, exit)?;
            sma.price.compute_ratio(from, close, exit)?;
        }
        self.pi_cycle.compute(from, lookback, close, exit)?;

        for (days, state) in self.ema_stored.iter_mut() {
            state.compute_rolling_ema(
                from,
                &priced_starts(lookback.start_vec(days)),
                close,
                exit,
            )?;
        }
        for (_, ema) in self.ema.iter_mut() {
            ema.compute_ratio(from, close, exit)?;
        }

        Ok(())
    }
}
