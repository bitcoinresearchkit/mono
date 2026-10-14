use bitview_cohort::AgeRange;
use bitview_compute::{
    CohortAccounting, Inputs, collect_age_range, weighted_age_aggregates, write_chunks,
    write_locked,
};
use bitview_distribution::WeightedCohortSources;
use bitview_plugin_age::{AccountingSources, Vecs as AgeVecs};
use bitview_plugin_indexer::Indexer;
use bitview_primitives::BoundedRatio;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;
use vecdb::{AnyVec, ReadableVec};

use super::{super::AgeRangeVecs, Vecs};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        age: &AgeVecs,
        age_range: &AgeRangeVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let accounting = age.accounting_sources();
        let weights = age_range.wakefulness_sources();
        compute_sources(
            &mut self.sources,
            starting_height,
            &accounting,
            &weights,
            exit,
        )
    }
}

fn compute_sources<W>(
    sources: &mut WeightedCohortSources,
    starting_height: Height,
    accounting: &AccountingSources<'_>,
    weights: &AgeRange<&W>,
    exit: &Exit,
) -> Result<()>
where
    W: ReadableVec<Height, BoundedRatio>,
{
    let mut inputs = Inputs::new(
        accounting.version(),
        weights.iter().map(|vec| *vec as &dyn AnyVec),
    );
    inputs.len = inputs.len.min(accounting.min_len());
    let range = inputs.prepare(
        sources.iter_any_stored_mut().collect::<Vec<_>>(),
        usize::from(starting_height),
        exit,
    )?;

    let mut accounting_batch = CohortAccounting::default();
    let mut weight_batch = AgeRange::default();
    for chunk in write_chunks(range) {
        accounting.collect_into(chunk.start, chunk.end, &mut accounting_batch);
        collect_age_range(weights, &mut weight_batch, chunk.start, chunk.end);
        for offset in 0..chunk.len() {
            sources.push(&weighted_age_aggregates(|id| {
                accounting_batch.weighted(id, offset, id.select(&weight_batch)[offset])
            }));
        }
        write_locked(sources.iter_any_stored_mut(), exit)?;
    }

    Ok(())
}
