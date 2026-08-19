use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    blocks: &bitview_plugin_blocks::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, blocks, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        blocks: &bitview_plugin_blocks::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let window_starts = blocks.lookback.window_starts();

        self.total
            .compute_rest(starting_height, &window_starts, exit)?;
        Ok(())
    }
}
