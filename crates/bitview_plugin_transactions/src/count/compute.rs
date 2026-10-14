use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::LookbackVecs;
use brk_error::Result;
use brk_exit::Exit;

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        lookback: &LookbackVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        let window_starts = lookback.window_starts();
        self.0.compute(starting_height, &window_starts, exit)?;

        Ok(())
    }
}
