use bitview_cohort::AgeRange;
use bitview_compute::{
    CohortAccounting, Inputs, collect_age_range, weighted_age_aggregates, write_chunks,
    write_locked,
};
use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_indexer::Indexer;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use vecdb::AnyVec;

use super::{super::AgeRangeVecs, Vecs};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        age: &AgeVecs,
        age_range: &AgeRangeVecs,
        exit: &Exit,
    ) -> Result<()> {
        let accounting = age.accounting_sources();
        let weights = age_range.wakefulness_sources();
        let mut inputs = Inputs::new(
            accounting.version(),
            weights.iter().map(|vec| *vec as &dyn AnyVec),
        );
        inputs.len = inputs.len.min(accounting.min_len());
        let range = inputs.prepare(
            self.sources.iter_any_stored_mut().collect::<Vec<_>>(),
            usize::from(indexer.compute_start().height),
            exit,
        )?;

        let mut accounting_batch = CohortAccounting::default();
        let mut weight_batch = AgeRange::default();
        for chunk in write_chunks(range) {
            accounting.collect_into(chunk.start, chunk.end, &mut accounting_batch);
            collect_age_range(&weights, &mut weight_batch, chunk.start, chunk.end);
            for offset in 0..chunk.len() {
                self.sources.push(&weighted_age_aggregates(|id| {
                    accounting_batch.weighted(id, offset, id.select(&weight_batch)[offset])
                }));
            }
            write_locked(self.sources.iter_any_stored_mut(), exit)?;
        }

        Ok(())
    }
}
