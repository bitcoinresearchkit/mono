use bitview_compute::compute_rolling_extrema_from_starts;
use bitview_plugin_mappings::LookbackVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_vecs::priced_starts;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        prices: &PriceVecs,
        lookback: &LookbackVecs,
        starting_height: Height,
        exit: &Exit,
    ) -> Result<()> {
        let price = &prices.spot.cents.height;

        let [week, month, year] = &mut self.extrema;
        for (min_vec, max_vec, starts, retained) in [
            (
                &mut self.min._1w.cents.height,
                &mut self.max._1w.cents.height,
                priced_starts(lookback.start_vec(7)),
                week,
            ),
            (
                &mut self.min._1m.cents.height,
                &mut self.max._1m.cents.height,
                priced_starts(lookback.start_vec(30)),
                month,
            ),
            (
                &mut self.min._1y.cents.height,
                &mut self.max._1y.cents.height,
                priced_starts(lookback.start_vec(365)),
                year,
            ),
        ] {
            compute_rolling_extrema_from_starts(
                min_vec,
                max_vec,
                starting_height,
                &starts,
                price,
                retained,
                exit,
            )?;
        }

        Ok(())
    }
}
