use bitview_compute::compute_rolling_extrema_from_starts;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::LookbackVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use brk_error::Result;
use brk_exit::Exit;

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        prices: &PriceVecs,
        lookback: &LookbackVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.compute_start().height;
        let price = &prices.spot.cents.height;

        let [week, month, year] = &mut self.extrema;
        for (min_vec, max_vec, starts, retained) in [
            (
                &mut self.min._1w.cents.height,
                &mut self.max._1w.cents.height,
                lookback.start_vec(7),
                week,
            ),
            (
                &mut self.min._1m.cents.height,
                &mut self.max._1m.cents.height,
                lookback.start_vec(30),
                month,
            ),
            (
                &mut self.min._1y.cents.height,
                &mut self.max._1y.cents.height,
                lookback.start_vec(365),
                year,
            ),
        ] {
            compute_rolling_extrema_from_starts(
                min_vec,
                max_vec,
                starting_height,
                starts,
                price,
                retained,
                exit,
            )?;
        }

        Ok(())
    }
}
