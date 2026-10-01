use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_indexer::Indexer;
use brk_error::Result;
use brk_exit::Exit;

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        blocks: &BlocksVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let window_starts = blocks.lookback.window_starts();

        self.compute_rest(starting_height, &window_starts, exit)?;

        Ok(())
    }
}
