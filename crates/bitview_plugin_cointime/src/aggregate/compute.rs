use std::iter;

use bitview_cohort::{AgeRange, ByTerm};
use bitview_compute::{
    CohortAccounting, WeightedCohortAggregates, WeightedCohortState, collect_age_range,
    prepare_computed,
};
use bitview_plugin_distribution_age::{AccountingSources, Vecs as AgeVecs};
use bitview_plugin_indexer::Indexer;
use bitview_vecs::PerBlock;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{BoundedRatio, Height, Version};
use vecdb::{AnyStoredVec, CachePolicy, EagerVec, PcoVec, ReadableVec, WritableVec};

use super::{super::AgeRangeVecs, Sources, Vecs};

const WRITE_INTERVAL: usize = 10_000;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    distribution_age: &AgeVecs,
    age_range: &mut AgeRangeVecs,
    all_supply_in_loss_share: &mut PerBlock<BoundedRatio>,
    exit: &Exit,
) -> Result<()> {
    let starting_height = indexer.safe_lengths().height;
    let accounting = distribution_age.accounting_sources();
    let weights = AgeRange::from_fn(|id| id.select(&age_range.activity_sources));

    vecs.sources.compute_primary(
        starting_height,
        &accounting,
        &weights,
        &mut all_supply_in_loss_share.height,
        exit,
    )
}

impl Sources {
    #[allow(clippy::too_many_arguments)]
    fn compute_primary<W>(
        &mut self,
        starting_height: Height,
        accounting: &AccountingSources<'_>,
        weights: &AgeRange<&W>,
        all_supply_in_loss_share: &mut EagerVec<PcoVec<Height, BoundedRatio, impl CachePolicy>>,
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
            self.primary_vecs_mut()
                .chain(iter::once(
                    all_supply_in_loss_share as &mut dyn AnyStoredVec,
                ))
                .collect::<Vec<_>>(),
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
                let WeightedCohortAggregates {
                    terms,
                    under_4m,
                    under_6m,
                    over_4m,
                    over_6m,
                } = WeightedCohortAggregates::from_fn(|id| {
                    accounting_batch.weighted(id, offset, id.select(&weight_batch)[offset])
                });
                self.under_4m_awake_price.push(under_4m.realized_price());
                self.under_4m_awake_capitalized_price
                    .push(under_4m.capitalized_price.value());
                self.under_6m_awake_price.push(under_6m.realized_price());
                self.under_6m_awake_capitalized_price
                    .push(under_6m.capitalized_price.value());
                self.over_4m_awake_price.push(over_4m.realized_price());
                self.over_4m_awake_capitalized_price
                    .push(over_4m.capitalized_price.value());
                self.over_6m_awake_price.push(over_6m.realized_price());
                self.over_6m_awake_capitalized_price
                    .push(over_6m.capitalized_price.value());
                let all = terms.short.merged(terms.long);
                all_supply_in_loss_share.push(all.supply_in_loss.value());
                self.push(terms, all);
            }

            {
                let _lock = exit.lock();
                for vec in self.primary_vecs_mut() {
                    vec.write()?;
                }
                all_supply_in_loss_share.write()?;
            }
            chunk_start = chunk_end;
        }

        Ok(())
    }

    fn push(&mut self, terms: ByTerm<WeightedCohortState>, all: WeightedCohortState) {
        for (target, value) in [
            (&mut self.awake_supply.all, all.weighted_supply),
            (&mut self.awake_supply.sth, terms.short.weighted_supply),
            (&mut self.awake_supply.lth, terms.long.weighted_supply),
            (&mut self.dormant_supply.all, all.complement_supply),
            (&mut self.dormant_supply.sth, terms.short.complement_supply),
            (&mut self.dormant_supply.lth, terms.long.complement_supply),
        ] {
            target.push(value);
        }
        for (target, value) in [
            (&mut self.awake_cap.all, all.weighted_cap),
            (&mut self.awake_cap.sth, terms.short.weighted_cap),
            (&mut self.awake_cap.lth, terms.long.weighted_cap),
            (&mut self.awake_price.all, all.realized_price()),
            (&mut self.awake_price.sth, terms.short.realized_price()),
            (&mut self.awake_price.lth, terms.long.realized_price()),
            (
                &mut self.awake_capitalized_price.all,
                all.capitalized_price.value(),
            ),
            (
                &mut self.awake_capitalized_price.sth,
                terms.short.capitalized_price.value(),
            ),
            (
                &mut self.awake_capitalized_price.lth,
                terms.long.capitalized_price.value(),
            ),
        ] {
            target.push(value);
        }
        self.supply_in_loss_share
            .short
            .push(terms.short.supply_in_loss.value());
        self.supply_in_loss_share
            .long
            .push(terms.long.supply_in_loss.value());
    }

    fn primary_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        [
            &mut self.over_4m_awake_price as &mut dyn AnyStoredVec,
            &mut self.over_4m_awake_capitalized_price,
            &mut self.over_6m_awake_price,
            &mut self.over_6m_awake_capitalized_price,
            &mut self.under_4m_awake_price,
            &mut self.under_4m_awake_capitalized_price,
            &mut self.under_6m_awake_price,
            &mut self.under_6m_awake_capitalized_price,
            &mut self.awake_supply.all,
            &mut self.awake_supply.sth,
            &mut self.awake_supply.lth,
            &mut self.dormant_supply.all,
            &mut self.dormant_supply.sth,
            &mut self.dormant_supply.lth,
            &mut self.awake_cap.all,
            &mut self.awake_cap.sth,
            &mut self.awake_cap.lth,
            &mut self.awake_price.all,
            &mut self.awake_price.sth,
            &mut self.awake_price.lth,
            &mut self.awake_capitalized_price.all,
            &mut self.awake_capitalized_price.sth,
            &mut self.awake_capitalized_price.lth,
            &mut self.supply_in_loss_share.short,
            &mut self.supply_in_loss_share.long,
        ]
        .into_iter()
    }
}
