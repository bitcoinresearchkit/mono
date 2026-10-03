use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use brk_error::Result;
use brk_exit::Exit;

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();

        self.weight.derive_from(
            mappings,
            &starting_lengths,
            &indexer.vecs().transactions.first_tx_index,
            &indexer.vecs().transactions.weight,
            exit,
        )?;

        Ok(())
    }
}
