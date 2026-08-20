use brk_error::Result;

use bitview_cohort::{AgeRange, AgeRangeId, ByTerm, TERM_FILTERS, UTXOAggregate};
use bitview_plugin_indexer::Indexer;
use brk_types::{Cents, Height, Sats, StoredF64, Version};
use vecdb::{AnyStoredVec, AnyVec, ColumnId, Exit, ReadableVec, WritableVec};

use super::{Sources, Vecs};
use bitview_compute::{
    PerBlock, WeightedCohortState, db_utils::validate_any_computed_version_or_reset,
};

const WRITE_INTERVAL: usize = 10_000;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    distribution: &bitview_plugin_distribution::Vecs,
    age_range: &super::super::AgeRangeVecs,
    all_supply_in_loss_share: &mut PerBlock<StoredF64>,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(
        indexer,
        distribution,
        age_range,
        all_supply_in_loss_share,
        exit,
    )
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        distribution: &bitview_plugin_distribution::Vecs,
        age_range: &super::super::AgeRangeVecs,
        all_supply_in_loss_share: &mut PerBlock<StoredF64>,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let supplies = AgeRange::from_fn(|id| {
            &id.select(&distribution.cohorts.supply.total.cohorts.age.range)
                .sats
                .height
        });
        let loss_supplies = AgeRange::from_fn(|id| {
            &id.select(&distribution.cohorts.supply.in_loss.cohorts.age.range)
                .sats
                .height
        });
        let realized_caps = AgeRange::from_fn(|id| {
            &id.select(&distribution.cohorts.realized.cap.cohorts.age.range)
                .cents
                .height
        });
        let weights = &age_range.activity.height;

        self.compute_primary(
            starting_height,
            &supplies,
            &loss_supplies,
            &realized_caps,
            weights,
            all_supply_in_loss_share,
            exit,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_primary<S, L, C, W>(
        &mut self,
        starting_height: Height,
        supplies: &AgeRange<&S>,
        loss_supplies: &AgeRange<&L>,
        realized_caps: &AgeRange<&C>,
        weights: &W,
        all_supply_in_loss_share: &mut PerBlock<StoredF64>,
        exit: &Exit,
    ) -> Result<()>
    where
        S: ReadableVec<Height, Sats>,
        L: ReadableVec<Height, Sats>,
        C: ReadableVec<Height, Cents>,
        W: ReadableVec<Height, AgeRange<StoredF64>>,
    {
        let source_version = Version::combine_all(
            supplies
                .iter()
                .map(|vec| vec.version())
                .chain(loss_supplies.iter().map(|vec| vec.version()))
                .chain(realized_caps.iter().map(|vec| vec.version()))
                .chain(std::iter::once(weights.version())),
        );

        for vec in self.primary_vecs_mut() {
            validate_any_computed_version_or_reset(vec, source_version)?;
        }
        let all_supply_in_loss_share = &mut all_supply_in_loss_share.height;
        validate_any_computed_version_or_reset(all_supply_in_loss_share, source_version)?;

        let start = self
            .primary_vecs_mut()
            .map(|vec| vec.len())
            .chain(std::iter::once(all_supply_in_loss_share.len()))
            .min()
            .unwrap_or_default()
            .min(usize::from(starting_height));

        for vec in self.primary_vecs_mut() {
            vec.any_truncate_if_needed_at(start)?;
        }
        all_supply_in_loss_share.truncate_if_needed_at(start)?;

        let source_end = supplies
            .iter()
            .map(|vec| vec.len())
            .chain(loss_supplies.iter().map(|vec| vec.len()))
            .chain(realized_caps.iter().map(|vec| vec.len()))
            .chain(std::iter::once(weights.len()))
            .min()
            .unwrap_or_default();

        let mut chunk_start = start;
        while chunk_start < source_end {
            let chunk_end = (chunk_start + WRITE_INTERVAL).min(source_end);
            let supply_batches = AgeRange::from_fn(|id| {
                id.select(supplies).collect_range_at(chunk_start, chunk_end)
            });
            let loss_batches = AgeRange::from_fn(|id| {
                id.select(loss_supplies)
                    .collect_range_at(chunk_start, chunk_end)
            });
            let cap_batches = AgeRange::from_fn(|id| {
                id.select(realized_caps)
                    .collect_range_at(chunk_start, chunk_end)
            });
            let weight_batch = weights.collect_range_at(chunk_start, chunk_end);

            for (offset, weights) in weight_batch.iter().enumerate() {
                let mut terms = ByTerm::<WeightedCohortState>::default();
                for &id in AgeRangeId::ALL {
                    let term = if TERM_FILTERS.short.includes(id.filter()) {
                        &mut terms.short
                    } else {
                        &mut terms.long
                    };
                    term.add(
                        id.select(&supply_batches)[offset],
                        id.select(&loss_batches)[offset],
                        id.select(&cap_batches)[offset],
                        *id.get(weights),
                    );
                }
                let all = terms.short.merged(terms.long);
                all_supply_in_loss_share.push(all.supply_in_loss.value());
                self.sources.push(terms, all);
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

    fn primary_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.sources.primary_vecs_mut().into_iter()
    }
}

impl Sources {
    fn push(&mut self, terms: ByTerm<WeightedCohortState>, all: WeightedCohortState) {
        self.awake_supply.push(ByTerm {
            short: terms.short.weighted_supply,
            long: terms.long.weighted_supply,
        });
        self.dormant_supply.push(ByTerm {
            short: terms.short.complement_supply,
            long: terms.long.complement_supply,
        });
        self.awake_cap.push(ByTerm {
            short: terms.short.weighted_cap,
            long: terms.long.weighted_cap,
        });
        self.awake_price.push(UTXOAggregate {
            all: all.realized_price(),
            sth: terms.short.realized_price(),
            lth: terms.long.realized_price(),
        });
        self.supply_in_loss_share.push(ByTerm {
            short: terms.short.supply_in_loss.value(),
            long: terms.long.supply_in_loss.value(),
        });
    }

    fn primary_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 5] {
        [
            &mut self.awake_supply,
            &mut self.dormant_supply,
            &mut self.awake_cap,
            &mut self.awake_price,
            &mut self.supply_in_loss_share,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_states_merge_into_all() {
        let mut sth = WeightedCohortState::default();
        sth.add(
            Sats::from(100_u64),
            Sats::from(20_u64),
            Cents::from(1_000_u64),
            StoredF64::from(0.3),
        );
        let mut lth = WeightedCohortState::default();
        lth.add(
            Sats::from(200_u64),
            Sats::from(50_u64),
            Cents::from(3_000_u64),
            StoredF64::from(0.4),
        );

        let all = sth.merged(lth);

        assert_eq!(
            all.weighted_supply,
            sth.weighted_supply + lth.weighted_supply
        );
        assert_eq!(
            all.complement_supply,
            sth.complement_supply + lth.complement_supply
        );
        assert_eq!(all.weighted_cap, sth.weighted_cap + lth.weighted_cap);
    }

    #[test]
    fn awake_and_dormant_supply_are_independently_floored() {
        let supply = Sats::from(123_456_789_u64);
        let weight = StoredF64::from(0.321);
        let mut state = WeightedCohortState::default();

        state.add(supply, Sats::ZERO, Cents::ZERO, weight);

        let sum = state.weighted_supply + state.complement_supply;
        assert!(sum <= supply);
        assert!(supply - sum <= Sats::from(1_u64));
    }
}
