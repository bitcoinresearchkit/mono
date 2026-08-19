use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;

pub trait Compute {
    fn compute(
        &mut self,
        indexer: &Indexer,
        lookback: &crate::LookbackVecs,
        exit: &Exit,
    ) -> Result<()>;
}

impl Compute for Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        lookback: &crate::LookbackVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let window_starts = lookback.window_starts();

        self.vbytes.compute(starting_height, &window_starts, exit)?;

        self.size.compute(
            starting_height,
            &window_starts,
            &indexer.vecs().blocks.total,
            exit,
        )?;

        Ok(())
    }
}
