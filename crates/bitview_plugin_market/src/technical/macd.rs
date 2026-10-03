use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use brk_error::Result;
use brk_exit::Exit;

use super::MacdChain;

impl MacdChain {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        blocks: &BlocksVecs,
        prices: &PriceVecs,
        fast_days: usize,
        slow_days: usize,
        signal_days: usize,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let close = &prices.spot.usd.height;
        let ws_fast = blocks.lookback.start_vec(fast_days);
        let ws_slow = blocks.lookback.start_vec(slow_days);
        let ws_signal = blocks.lookback.start_vec(signal_days);

        self.ema_fast
            .height
            .compute_rolling_ema(starting_height, ws_fast, close, exit)?;

        self.ema_slow
            .height
            .compute_rolling_ema(starting_height, ws_slow, close, exit)?;

        self.line.height.compute_subtract(
            starting_height,
            &self.ema_fast.height,
            &self.ema_slow.height,
            exit,
        )?;

        self.signal.height.compute_rolling_ema(
            starting_height,
            ws_signal,
            &self.line.height,
            exit,
        )?;

        self.histogram.height.compute_subtract(
            starting_height,
            &self.line.height,
            &self.signal.height,
            exit,
        )?;

        Ok(())
    }
}
