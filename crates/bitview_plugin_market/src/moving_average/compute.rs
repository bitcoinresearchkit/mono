use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_exit::Exit;

use super::Vecs;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    blocks: &bitview_plugin_blocks::Vecs,
    prices: &bitview_plugin_price::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, blocks, prices, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        blocks: &bitview_plugin_blocks::Vecs,
        prices: &bitview_plugin_price::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();
        let close = &prices.spot.cents.height;

        self.ema.height.compute_rolling_ema_columns(
            starting_lengths.height,
            |period| blocks.lookback.start_vec(period.days()),
            close,
            exit,
        )?;

        Ok(())
    }
}
