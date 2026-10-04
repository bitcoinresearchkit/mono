use bitview_plugin_indexer::Indexer;
use brk_error::Result;
use brk_exit::Exit;

use super::Vecs;
use crate::LookbackVecs;

impl Vecs {
    pub fn compute(
        &mut self,
        indexer: &Indexer,
        lookback: &LookbackVecs,
        exit: &Exit,
    ) -> Result<()> {
        self.weight.compute(
            indexer.safe_lengths().height,
            &lookback.window_starts(),
            &indexer.vecs().blocks.weight,
            exit,
        )
    }
}
