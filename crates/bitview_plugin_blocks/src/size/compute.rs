use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_exit::Exit;
use rayon::join;

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
        let Self { vbytes, size } = self;

        let (vbytes_result, size_result) = join(
            || vbytes.compute(starting_height, &window_starts, exit),
            || {
                size.compute(
                    starting_height,
                    &window_starts,
                    &indexer.vecs().blocks.total,
                    exit,
                )
            },
        );
        vbytes_result?;
        size_result?;

        Ok(())
    }
}
