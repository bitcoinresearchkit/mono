use brk_error::Result;

use brk_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    lookback: &bitview_plugin_blocks::LookbackVecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, lookback, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        lookback: &bitview_plugin_blocks::LookbackVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        let window_starts = lookback.window_starts();
        self.total.compute(starting_height, &window_starts, exit)?;

        Ok(())
    }
}
