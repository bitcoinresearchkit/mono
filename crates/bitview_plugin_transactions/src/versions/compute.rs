use bitview_plugin_indexer::Indexer;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::StoredU64;

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let source = &indexer.vecs().transaction_features.count;
        for (target, source) in [
            (&mut self.v1, &source.v1),
            (&mut self.v2, &source.v2),
            (&mut self.v3, &source.v3),
            (&mut self.other, &source.other_version),
        ] {
            target.compute_cumulative_transformed(
                starting_height,
                source,
                StoredU64::from,
                exit,
            )?;
        }
        Ok(())
    }
}
