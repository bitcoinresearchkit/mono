use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_types::PartsPerMillion32;
use vecdb::{Exit, VecIndex};

use super::Vecs;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    prices: &bitview_plugin_price::Vecs,
    blocks: &bitview_plugin_blocks::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, prices, blocks, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        prices: &bitview_plugin_price::Vecs,
        blocks: &bitview_plugin_blocks::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let price = &prices.spot.cents.height;

        for (min_vec, max_vec, starts) in [
            (
                &mut self.min._1w.cents.height,
                &mut self.max._1w.cents.height,
                blocks.lookback._1w.lazy(),
            ),
            (
                &mut self.min._2w.cents.height,
                &mut self.max._2w.cents.height,
                &blocks.lookback._2w,
            ),
            (
                &mut self.min._1m.cents.height,
                &mut self.max._1m.cents.height,
                blocks.lookback._1m.lazy(),
            ),
            (
                &mut self.min._1y.cents.height,
                &mut self.max._1y.cents.height,
                blocks.lookback._1y.lazy(),
            ),
        ] {
            min_vec.compute_rolling_min_from_starts(starting_height, starts, price, exit)?;
            max_vec.compute_rolling_max_from_starts(starting_height, starts, price, exit)?;
        }

        // 2w rolling sum of true range
        self.true_range_sum_2w.height.compute_rolling_sum(
            starting_height,
            &blocks.lookback._2w,
            &self.true_range.height,
            exit,
        )?;

        self.choppiness_index_2w.ppm.height.compute_transform4(
            starting_height,
            &self.true_range_sum_2w.height,
            &self.max._2w.cents.height,
            &self.min._2w.cents.height,
            &blocks.lookback._2w,
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
