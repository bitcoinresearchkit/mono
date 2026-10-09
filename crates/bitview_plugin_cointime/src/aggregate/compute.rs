use std::iter;

use bitview_cohort::{AgeAggregate, AgeAggregateId, AgeRange};
use bitview_compute::{
    CohortAccounting, WeightedCohortState, collect_age_range, prepare_computed,
    weighted_age_aggregates,
};
use bitview_plugin_age::{AccountingSources, Vecs as AgeVecs};
use bitview_plugin_indexer::Indexer;
use bitview_primitives::BoundedRatio;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use vecdb::{AnyStoredVec, ReadableVec, WritableVec};

use super::{super::AgeRangeVecs, Sources, Vecs};

const WRITE_INTERVAL: usize = 10_000;

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
        self.sources
            .compute(starting_height, &accounting, &weights, exit)
    }
}

impl Sources {
    fn compute<W>(
        &mut self,
        starting_height: Height,
        accounting: &AccountingSources<'_>,
        weights: &AgeRange<&W>,
        exit: &Exit,
    ) -> Result<()>
    where
        W: ReadableVec<Height, BoundedRatio>,
    {
        let source_version = Version::combine_all(
            iter::once(accounting.version()).chain(weights.iter().map(|vec| vec.version())),
        );

        let aggregate_end = weights
            .iter()
            .map(|vec| vec.len())
            .min()
            .unwrap_or_default()
            .min(accounting.min_len());

        let start = prepare_computed(
            self.vecs_mut().collect::<Vec<_>>(),
            source_version,
            usize::from(starting_height).min(aggregate_end),
            exit,
        )?;

        let mut accounting_batch = CohortAccounting::default();
        let mut weight_batch = AgeRange::default();
        let mut chunk_start = start;
        while chunk_start < aggregate_end {
            let chunk_end = (chunk_start + WRITE_INTERVAL).min(aggregate_end);
            accounting.collect_into(chunk_start, chunk_end, &mut accounting_batch);
            collect_age_range(weights, &mut weight_batch, chunk_start, chunk_end);

            for offset in 0..chunk_end - chunk_start {
                self.push(weighted_age_aggregates(|id| {
                    accounting_batch.weighted(id, offset, id.select(&weight_batch)[offset])
                }));
            }

            {
                let _lock = exit.lock();
                for vec in self.vecs_mut() {
                    vec.write()?;
                }
            }
            chunk_start = chunk_end;
        }

        Ok(())
    }

    fn push(&mut self, states: AgeAggregate<WeightedCohortState>) {
        for &id in AgeAggregateId::ALL {
            let state = id.select(&states);
            id.select_mut(&mut self.awake_supply)
                .push(state.weighted_supply);
            id.select_mut(&mut self.dormant_supply)
                .push(state.complement_supply);
            id.select_mut(&mut self.awake_realized_cap)
                .push(state.weighted_cap);
            id.select_mut(&mut self.awake_realized_price)
                .push(state.realized_price());
            id.select_mut(&mut self.awake_capitalized_price)
                .push(state.capitalized_price.value());
            id.select_mut(&mut self.awake_supply_in_loss_share)
                .push(state.supply_in_loss.value());
        }
    }

    fn vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        let Self {
            awake_supply,
            dormant_supply,
            awake_realized_cap,
            awake_realized_price,
            awake_capitalized_price,
            awake_supply_in_loss_share,
        } = self;
        awake_supply
            .iter_mut()
            .map(|vec| vec as &mut dyn AnyStoredVec)
            .chain(
                dormant_supply
                    .iter_mut()
                    .map(|vec| vec as &mut dyn AnyStoredVec),
            )
            .chain(
                awake_realized_cap
                    .iter_mut()
                    .map(|vec| vec as &mut dyn AnyStoredVec),
            )
            .chain(
                awake_realized_price
                    .iter_mut()
                    .map(|vec| vec as &mut dyn AnyStoredVec),
            )
            .chain(
                awake_capitalized_price
                    .iter_mut()
                    .map(|vec| vec as &mut dyn AnyStoredVec),
            )
            .chain(
                awake_supply_in_loss_share
                    .iter_mut()
                    .map(|vec| vec as &mut dyn AnyStoredVec),
            )
    }
}
