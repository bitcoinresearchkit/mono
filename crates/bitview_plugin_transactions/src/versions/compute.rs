use bitview_plugin_indexer::Indexer;
use bitview_primitives::Count;
use brk_error::Result;
use brk_exit::Exit;

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let source = &indexer.vecs().transactions.versions;
        for (target, source) in [
            (&mut self.v1, &*source.v1.block),
            (&mut self.v2, &*source.v2.block),
            (&mut self.v3, &*source.v3.block),
            (&mut self.other, &*source.other.block),
        ] {
            target.compute_cumulative_transformed(starting_height, source, Count::from, exit)?;
        }
        Ok(())
    }
}
