use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::SigOps;

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        exit: &Exit,
    ) -> Result<()> {
        self.total.compute_cumulative_sum_from_indexes(
            indexer.safe_lengths().height,
            &indexer.vecs().transactions.first_tx_index,
            &mappings.height.tx_index_count,
            &*indexer.vecs().transactions.total_sigop_cost,
            SigOps::from,
            exit,
        )
    }
}
