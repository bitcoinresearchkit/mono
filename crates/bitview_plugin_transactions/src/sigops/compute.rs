use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_types::StoredU64;
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
        self.total.compute_cumulative_sum_from_indexes(
            indexer.safe_lengths().height,
            &indexer.vecs().transactions.first_tx_index,
            &indexes.height.tx_index_count,
            &indexer.vecs().transactions.total_sigop_cost,
            |value| StoredU64::from(u64::from(u32::from(value))),
            exit,
        )
    }
}
