use bitview_compute::ComputeRollingMedianFromStarts;
use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::StoredF64;

use super::{super::value, Vecs};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        blocks: &BlocksVecs,
        prices: &PriceVecs,
        value: &value::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        self.vocdd_median_1y.compute_rolling_median_from_starts(
            starting_height,
            &blocks.lookback._1y,
            &value.vocdd.block,
            exit,
        )?;

        self.hodl_bank.compute_cumulative_transformed_binary(
            starting_height,
            &prices.spot.usd.height,
            &self.vocdd_median_1y,
            |price, median| StoredF64::from(f64::from(price) - f64::from(median)),
            exit,
        )?;

        Ok(())
    }
}
