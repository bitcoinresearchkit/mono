use bitview_cohort::{AgeRange, ByTerm};
use bitview_compute::{AgeBand, MINIMUM_DURATION_DAYS, WeightedCohortAggregates};
use brk_types::{
    Bitcoin, BoundedRatio, Cents, CentsSats, CentsSquaredSats, Height, Sats, StoredF64, Timestamp,
};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use vecdb::{ReadableVec, VecValue};

use super::{PrimaryValues, aggregate::AggregateState, decay::DecayFit};
use crate::weights::horizon_mobilities;

pub(crate) struct PrimaryBatch {
    timestamps: Vec<Timestamp>,
    transfer_volumes: AgeRange<Vec<Sats>>,
    coindays_created: AgeRange<Vec<StoredF64>>,
    supplies: AgeRange<Vec<Sats>>,
    loss_supplies: AgeRange<Vec<Sats>>,
    realized_caps: AgeRange<Vec<Cents>>,
    cap_raw: AgeRange<Vec<CentsSats>>,
    capitalized_cap_raw: AgeRange<Vec<CentsSquaredSats>>,
}

impl PrimaryBatch {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn collect(
        timestamps: &impl ReadableVec<Height, Timestamp>,
        transfer_volumes: &AgeRange<&impl ReadableVec<Height, Sats>>,
        coindays_created: &AgeRange<&impl ReadableVec<Height, StoredF64>>,
        supplies: &AgeRange<&impl ReadableVec<Height, Sats>>,
        loss_supplies: &AgeRange<&impl ReadableVec<Height, Sats>>,
        realized_caps: &AgeRange<&impl ReadableVec<Height, Cents>>,
        cap_raw: &AgeRange<&impl ReadableVec<Height, CentsSats>>,
        capitalized_cap_raw: &AgeRange<&impl ReadableVec<Height, CentsSquaredSats>>,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            timestamps: timestamps.collect_range_at(start, end),
            transfer_volumes: Self::collect_age_range(transfer_volumes, start, end),
            coindays_created: Self::collect_age_range(coindays_created, start, end),
            supplies: Self::collect_age_range(supplies, start, end),
            loss_supplies: Self::collect_age_range(loss_supplies, start, end),
            realized_caps: Self::collect_age_range(realized_caps, start, end),
            cap_raw: Self::collect_age_range(cap_raw, start, end),
            capitalized_cap_raw: Self::collect_age_range(capitalized_cap_raw, start, end),
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.timestamps.len()
    }

    fn collect_age_range<T, V>(sources: &AgeRange<&V>, start: usize, end: usize) -> AgeRange<Vec<T>>
    where
        T: VecValue,
        V: ReadableVec<Height, T>,
    {
        AgeRange::par_from_fn(|id| id.select(sources).collect_range_at(start, end))
    }

    pub(crate) fn primary_values_batch(
        &self,
        genesis_timestamp: Timestamp,
        bounds: &AgeRange<AgeBand>,
    ) -> Vec<PrimaryValues> {
        (0..self.len())
            .into_par_iter()
            .map(|offset| self.primary_values(offset, genesis_timestamp, bounds))
            .collect()
    }

    fn primary_values(
        &self,
        offset: usize,
        genesis_timestamp: Timestamp,
        bounds: &AgeRange<AgeBand>,
    ) -> PrimaryValues {
        let hazards = AgeRange::from_fn(|id| {
            Self::spending_rate(
                id.select(&self.transfer_volumes)[offset],
                id.select(&self.coindays_created)[offset],
            )
        });
        let network_age = self.timestamps[offset]
            .difference_in_days_between_float(genesis_timestamp)
            .max(MINIMUM_DURATION_DAYS);
        let exposures = DecayFit::exposures(&hazards, network_age, bounds);
        let mobilities =
            AgeRange::from_fn(|id| BoundedRatio::from(AgeBand::mobility(*id.select(&exposures))));
        let horizon_mobilities = horizon_mobilities(&hazards, bounds);
        let mut terms = ByTerm::<AggregateState>::default();
        let aggregates = WeightedCohortAggregates::from_fn(|id| {
            let mobility = *id.select(&mobilities);
            let total_supply = id.select(&self.supplies)[offset];
            let total_cap = id.select(&self.realized_caps)[offset];
            let loss_supply = id.select(&self.loss_supplies)[offset];

            let mut contribution = AggregateState::default();
            contribution.weighted.capitalized_price.add(
                id.select(&self.cap_raw)[offset],
                id.select(&self.capitalized_cap_raw)[offset],
                mobility,
            );
            contribution.add(
                total_supply,
                loss_supply,
                total_cap,
                mobility,
                &horizon_mobilities,
                id,
            );
            terms.get_mut(id.term()).merge_horizons(&contribution);
            contribution.weighted
        });
        terms.short.weighted = aggregates.terms.short;
        terms.long.weighted = aggregates.terms.long;

        PrimaryValues {
            spending_rate: AgeRange::from_fn(|id| StoredF64::from(*id.select(&hazards))),
            spending_exposure: AgeRange::from_fn(|id| StoredF64::from(*id.select(&exposures))),
            mobility: mobilities,
            under_4m: aggregates.under_4m,
            under_6m: aggregates.under_6m,
            over_4m: aggregates.over_4m,
            over_6m: aggregates.over_6m,
            terms,
        }
    }

    #[inline]
    fn spending_rate(transfer_volume: Sats, coindays_created: StoredF64) -> f64 {
        let exposure = f64::from(coindays_created);
        if exposure > 0.0 {
            (f64::from(Bitcoin::from(transfer_volume)) / exposure).max(0.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitview_cohort::AgeRangeId;
    use bitview_compute::WeightedCohortState;

    #[test]
    fn mobility_is_the_complement_of_survival() {
        assert_eq!(AgeBand::mobility(0.0), 0.0);
        assert!((AgeBand::mobility(2.0_f64.ln()) - 0.5).abs() < 1e-12);
        assert!((AgeBand::mobility(1e-15) - 1e-15).abs() < 1e-27);
        assert!(AgeBand::mobility(1_000.0) < 1.0);
        assert_eq!(AgeBand::mobility(f64::INFINITY), 1.0 - 1e-12);
        assert_eq!(AgeBand::mobility(f64::NAN), 0.0);
    }

    #[test]
    fn mobile_and_immobile_supply_are_independently_floored() {
        let total_supply = Sats::from(123_456_789_u64);
        let total_cap = Cents::from(987_654_321_u64);
        let mobility = BoundedRatio::from(0.321);
        let mut state = WeightedCohortState::default();

        state.add(total_supply, Sats::ZERO, total_cap, mobility);

        let sum = state.weighted_supply + state.complement_supply;
        assert!(sum <= total_supply);
        assert!(total_supply - sum <= Sats::from(1_u64));
        assert!(state.weighted_cap <= total_cap);
    }

    #[test]
    fn primary_row_reuses_bounded_mobility_for_weighted_outputs() {
        let bounds = AgeBand::all();
        let supply = Sats::from(123_456_789_u64);
        let cap = Cents::from(987_654_321_u64);
        let batch = PrimaryBatch {
            timestamps: vec![Timestamp::from(20 * 365 * 86_400_u32)],
            transfer_volumes: AgeRange::from_fn(|id| {
                let age = id.select(&bounds).lower;
                vec![Sats::from((100_000_000.0 * (-age / 1_000.0).exp()) as u64)]
            }),
            coindays_created: AgeRange::from_fn(|_| vec![StoredF64::from(100.0)]),
            supplies: AgeRange::from_fn(|_| vec![supply]),
            loss_supplies: AgeRange::from_fn(|_| vec![Sats::from(10_u64)]),
            realized_caps: AgeRange::from_fn(|_| vec![cap]),
            cap_raw: AgeRange::from_fn(|_| {
                vec![CentsSats::new(cap.as_u128() * Sats::ONE_BTC_U128)]
            }),
            capitalized_cap_raw: AgeRange::from_fn(|_| {
                vec![CentsSquaredSats::new(
                    3_000 * cap.as_u128() * Sats::ONE_BTC_U128,
                )]
            }),
        };
        let values = batch.primary_values(0, Timestamp::ZERO, &bounds);
        let mut expected = WeightedCohortState::default();
        for &id in AgeRangeId::ALL {
            let raw = *id.select(&values.mobility);
            let exposure = f64::from(*id.select(&values.spending_exposure));
            assert_eq!(raw, BoundedRatio::from(AgeBand::mobility(exposure)));
            expected.add(supply, Sats::from(10_u64), cap, raw);
        }
        assert!(
            values
                .mobility
                .iter()
                .any(|value| *value != BoundedRatio::ZERO)
        );
        for state in [
            values.terms.short,
            values.terms.long,
            values.terms.short.merged(values.terms.long),
        ] {
            assert_eq!(state.weighted.capitalized_price.value(), Cents::new(3_000));
        }
        let all = values.terms.short.merged(values.terms.long).weighted;
        assert_eq!(all.weighted_supply, expected.weighted_supply);
        assert_eq!(all.complement_supply, expected.complement_supply);
        assert_eq!(all.weighted_cap, expected.weighted_cap);
    }

    #[test]
    fn age_threshold_prices_follow_bounded_mobility_and_exact_cutoffs() {
        let bounds = AgeBand::all();
        let batch = PrimaryBatch {
            timestamps: vec![Timestamp::from(20 * 365 * 86_400_u32)],
            transfer_volumes: AgeRange::from_fn(|id| {
                vec![Sats::from(100_000_000_u64 / (id.index() as u64 + 1))]
            }),
            coindays_created: AgeRange::from_fn(|_| vec![StoredF64::from(100.0)]),
            supplies: AgeRange::from_fn(|_| vec![Sats::from(100_000_000_u64)]),
            loss_supplies: AgeRange::from_fn(|_| vec![Sats::ZERO]),
            realized_caps: AgeRange::from_fn(|id| vec![Cents::new((id.index() as u64 + 1) * 100)]),
            cap_raw: AgeRange::from_fn(|id| {
                vec![CentsSats::new(
                    (id.index() as u128 + 1) * 100 * Sats::ONE_BTC_U128,
                )]
            }),
            capitalized_cap_raw: AgeRange::from_fn(|id| {
                vec![CentsSquaredSats::new(
                    ((id.index() as u128 + 1) * 100).pow(2) * Sats::ONE_BTC_U128,
                )]
            }),
        };
        let values = batch.primary_values(0, Timestamp::ZERO, &bounds);
        for (days, older, actual) in [
            (120, false, values.under_4m),
            (150, false, values.terms.short.weighted),
            (180, false, values.under_6m),
            (120, true, values.over_4m),
            (150, true, values.terms.long.weighted),
            (180, true, values.over_6m),
        ] {
            let mut expected = WeightedCohortState::default();
            for &id in AgeRangeId::ALL {
                let included = if older {
                    id.bounds().start >= days * 24
                } else {
                    id.bounds().end <= days * 24
                };
                if !included {
                    continue;
                }
                let weight = *id.select(&values.mobility);
                expected.add(
                    id.select(&batch.supplies)[0],
                    Sats::ZERO,
                    id.select(&batch.realized_caps)[0],
                    weight,
                );
                expected.capitalized_price.add(
                    id.select(&batch.cap_raw)[0],
                    id.select(&batch.capitalized_cap_raw)[0],
                    weight,
                );
            }
            assert_eq!(actual.realized_price(), expected.realized_price());
            assert_eq!(
                actual.capitalized_price.value(),
                expected.capitalized_price.value()
            );
        }
        assert_ne!(
            values.under_4m.realized_price(),
            values.under_6m.realized_price()
        );
        assert_ne!(
            values.under_4m.capitalized_price.value(),
            values.under_6m.capitalized_price.value()
        );
    }
}
