use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_types::StoredF64;
use vecdb::Exit;

use super::{super::value, Vecs};
use bitview_compute::algo::ComputeRollingMedianFromStarts;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    blocks: &bitview_plugin_blocks::Vecs,
    prices: &bitview_plugin_price::Vecs,
    value: &value::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, blocks, prices, value, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        blocks: &bitview_plugin_blocks::Vecs,
        prices: &bitview_plugin_price::Vecs,
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
