use brk_error::Result;
use brk_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;
use crate::{blocks, price};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        blocks: &blocks::Vecs,
        prices: &price::Vecs,
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
