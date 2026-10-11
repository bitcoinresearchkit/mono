use bitview_plugin_indexer::Indexer;
use bitview_primitives::Count;
use brk_error::Result;
use brk_exit::Exit;

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let source = &indexer.vecs().transactions.features;
        for (target, source) in [
            (&mut self.segwit, &*indexer.vecs().blocks.segwit_tx_count),
            (&mut self.p2tr.count, &*source.p2tr.block),
            (&mut self.annex.count, &*source.annex.block),
            (&mut self.sighash_all.count, &*source.sighash_all.block),
            (&mut self.sighash_none.count, &*source.sighash_none.block),
            (
                &mut self.sighash_single.count,
                &*source.sighash_single.block,
            ),
            (
                &mut self.sighash_default.count,
                &*source.sighash_default.block,
            ),
            (
                &mut self.sighash_anyone_can_pay.count,
                &*source.sighash_anyone_can_pay.block,
            ),
            (
                &mut self.explicitly_rbf.count,
                &*source.explicitly_rbf.block,
            ),
            (&mut self.dust_output.count, &*source.dust_output.block),
        ] {
            target.compute_cumulative_transformed(starting_height, source, Count::from, exit)?;
        }
        Ok(())
    }
}
