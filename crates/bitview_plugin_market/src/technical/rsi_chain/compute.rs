use bitview_compute::{ComputeRollingStats, compute_rolling_extrema_from_starts};
use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_indexer::Indexer;
use bitview_primitives::PartsPerMillion32;
use brk_error::Result;
use brk_exit::Exit;

use super::RsiChain;

impl RsiChain {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        blocks: &BlocksVecs,
        rma_days: usize,
        stoch_sma_days: usize,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let ws_rma = blocks.lookback.start_vec(rma_days);
        let ws_sma = blocks.lookback.start_vec(stoch_sma_days);

        self.average_gain.height.compute_rolling_rma(
            starting_height,
            ws_rma,
            &self.gains.height,
            exit,
        )?;

        self.average_loss.height.compute_rolling_rma(
            starting_height,
            ws_rma,
            &self.losses.height,
            exit,
        )?;

        self.rsi.ppm.height.compute_transform2(
            starting_height,
            &self.average_gain.height,
            &self.average_loss.height,
            |(h, g, l, ..)| {
                let sum = *g + *l;
                let rsi = if sum == 0.0 { 0.5 } else { *g / sum };
                (h, PartsPerMillion32::from(rsi as f64))
            },
            exit,
        )?;

        compute_rolling_extrema_from_starts(
            &mut self.rsi_min.ppm.height,
            &mut self.rsi_max.ppm.height,
            starting_height,
            ws_rma,
            &self.rsi.ppm.height,
            exit,
        )?;

        self.stoch_rsi.ppm.height.compute_transform3(
            starting_height,
            &self.rsi.ppm.height,
            &self.rsi_min.ppm.height,
            &self.rsi_max.ppm.height,
            |(h, r, mn, mx, ..)| {
                let range = f64::from(*mx) - f64::from(*mn);
                let stoch = if range == 0.0 {
                    PartsPerMillion32::ZERO
                } else {
                    PartsPerMillion32::from((f64::from(*r) - f64::from(*mn)) / range)
                };
                (h, stoch)
            },
            exit,
        )?;

        self.stoch_rsi_k.ppm.height.compute_rolling_average(
            starting_height,
            ws_sma,
            &self.stoch_rsi.ppm.height,
            exit,
        )?;

        self.stoch_rsi_d.ppm.height.compute_rolling_average(
            starting_height,
            ws_sma,
            &self.stoch_rsi_k.ppm.height,
            exit,
        )?;

        Ok(())
    }
}
