use brk_cohort::{AGE_RANGE_COUNT, AgeRange, AgeRangeId, ByTerm, TERM_FILTERS};
use brk_error::Result;
use brk_indexer::Indexer;
use brk_types::{Cents, Height, Sats, StoredF64, Version};
use vecdb::{AnyStoredVec, AnyVec, ColumnId, Exit, ReadableVec, WritableVec};

use super::{AllCohortVecs, StoredCohortVecs, Vecs};
use crate::{
    distribution,
    frameworks::WeightedCohortState,
    internal::{PerBlock, db_utils::validate_any_computed_version_or_reset},
};

const WRITE_INTERVAL: usize = 10_000;
const ALL_PRIMARY_VEC_COUNT: usize = 4;
const STORED_PRIMARY_VEC_COUNT: usize = 5;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        distribution: &distribution::Vecs,
        age_range: &super::super::AgeRangeVecs,
        all_supply_in_loss_share: &mut PerBlock<StoredF64>,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let supplies = AgeRange::from_fn(|id| {
            &id.select(&distribution.utxo_cohorts.age_range)
                .metrics
                .supply
                .total
                .sats
                .height
        });
        let loss_supplies = AgeRange::from_fn(|id| {
            &id.select(&distribution.utxo_cohorts.age_range)
                .metrics
                .supply
                .in_loss
                .sats
                .height
        });
        let realized_caps = AgeRange::from_fn(|id| {
            &id.select(&distribution.utxo_cohorts.age_range)
                .metrics
                .realized
                .cap
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
    fn compute_primary<S, C, W>(
        &mut self,
        starting_height: Height,
        supplies: &AgeRange<&S>,
        loss_supplies: &AgeRange<&S>,
        realized_caps: &AgeRange<&C>,
        weights: &W,
        all_supply_in_loss_share: &mut PerBlock<StoredF64>,
        exit: &Exit,
    ) -> Result<()>
    where
        S: ReadableVec<Height, Sats>,
        C: ReadableVec<Height, Cents>,
        W: ReadableVec<Height, [StoredF64; AGE_RANGE_COUNT]>,
    {
        let source_version: Version = supplies
            .iter()
            .map(|vec| vec.version())
            .chain(loss_supplies.iter().map(|vec| vec.version()))
            .chain(realized_caps.iter().map(|vec| vec.version()))
            .chain(std::iter::once(weights.version()))
            .sum();

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
                self.all.push(all);
                self.sth.push(terms.short);
                self.lth.push(terms.long);
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
        self.all
            .primary_vecs_mut()
            .into_iter()
            .chain(self.sth.primary_vecs_mut())
            .chain(self.lth.primary_vecs_mut())
    }
}

impl AllCohortVecs {
    fn push(&mut self, state: WeightedCohortState) {
        self.awake.supply.sats.height.push(state.weighted_supply);
        self.dormant
            .supply
            .sats
            .height
            .push(state.complement_supply);
        self.awake.cap.cents.height.push(state.weighted_cap);
        self.awake.price.cents.height.push(state.realized_price());
    }

    fn primary_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; ALL_PRIMARY_VEC_COUNT] {
        [
            &mut self.awake.supply.sats.height,
            &mut self.dormant.supply.sats.height,
            &mut self.awake.cap.cents.height,
            &mut self.awake.price.cents.height,
        ]
    }
}

impl StoredCohortVecs {
    fn push(&mut self, state: WeightedCohortState) {
        self.awake.supply.sats.height.push(state.weighted_supply);
        self.dormant
            .supply
            .sats
            .height
            .push(state.complement_supply);
        self.awake
            .supply_in_loss_share
            .height
            .push(state.supply_in_loss.value());
        self.awake.cap.cents.height.push(state.weighted_cap);
        self.awake.price.cents.height.push(state.realized_price());
    }

    fn primary_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; STORED_PRIMARY_VEC_COUNT] {
        [
            &mut self.awake.supply.sats.height,
            &mut self.dormant.supply.sats.height,
            &mut self.awake.supply_in_loss_share.height,
            &mut self.awake.cap.cents.height,
            &mut self.awake.price.cents.height,
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
