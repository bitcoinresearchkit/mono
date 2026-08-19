use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;
impl Vecs {
    pub fn compute(
        &mut self,
        indexer: &Indexer,
        blocks: &bitview_plugin_blocks::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let window_starts = blocks.lookback.window_starts();

        self.compute_rest(starting_height, &window_starts, exit)?;

        Ok(())
    }
}
