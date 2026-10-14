use bitview_compute::compute_rolling_extrema_from_starts;
use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use brk_error::Result;
use brk_exit::Exit;

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

        Ok(())
    }
}
