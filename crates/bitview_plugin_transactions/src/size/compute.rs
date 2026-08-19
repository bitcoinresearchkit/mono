use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    indexes: &bitview_plugin_indexes::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, indexes, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        indexes: &bitview_plugin_indexes::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();

        self.weight.derive_from(
            indexes,
            &starting_lengths,
            &indexer.vecs().transactions.first_tx_index,
            &indexer.vecs().transactions.weight,
            exit,
        )?;

        Ok(())
    }
}
