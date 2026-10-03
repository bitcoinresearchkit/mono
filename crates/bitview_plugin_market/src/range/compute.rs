use bitview_compute::{ComputeRollingStats, compute_rolling_extrema_from_starts};
use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::PartsPerMillion32;
use brk_error::Result;
use brk_exit::Exit;
use vecdb::VecIndex;

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        prices: &PriceVecs,
        blocks: &BlocksVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let price = &prices.spot.cents.height;

        for (min_vec, max_vec, starts) in [
            (
                &mut self.min._1w.cents.height,
                &mut self.max._1w.cents.height,
                blocks.lookback.start_vec(7),
            ),
            (
                &mut self.min._2w.cents.height,
                &mut self.max._2w.cents.height,
                blocks.lookback.start_vec(14),
            ),
            (
                &mut self.min._1m.cents.height,
                &mut self.max._1m.cents.height,
                blocks.lookback.start_vec(30),
            ),
            (
                &mut self.min._1y.cents.height,
                &mut self.max._1y.cents.height,
                blocks.lookback.start_vec(365),
            ),
        ] {
            compute_rolling_extrema_from_starts(
                min_vec,
                max_vec,
                starting_height,
                starts,
                price,
                exit,
            )?;
        }

        // 2w rolling sum of true range
        self.true_range_sum_2w.height.compute_rolling_sum(
            starting_height,
            blocks.lookback.start_vec(14),
            &self.true_range.height,
            exit,
        )?;

        self.choppiness_index_2w.ppm.height.compute_transform4(
            starting_height,
            &self.true_range_sum_2w.height,
            &self.max._2w.cents.height,
            &self.min._2w.cents.height,
            blocks.lookback.start_vec(14),
            |(h, tr_sum, max, min, window_start, ..)| {
                let range = f64::from(max) - f64::from(min);
                let n = (h.to_usize() - window_start.to_usize() + 1) as f32;
                let ci = if range > 0.0 && n > 1.0 {
                    PartsPerMillion32::from(
                        (*tr_sum / range as f32).log10() as f64 / n.log10() as f64,
                    )
                } else {
                    PartsPerMillion32::ZERO
                };
                (h, ci)
            },
            exit,
        )?;

        Ok(())
    }
}
