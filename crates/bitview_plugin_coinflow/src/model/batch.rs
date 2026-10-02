use bitview_cohort::AgeRange;
use bitview_compute::{
    AgeBand, CohortAccounting, MINIMUM_DURATION_DAYS, WeightedCohortAggregates, collect_age_range,
};
use bitview_plugin_distribution_age::AccountingSources;
use brk_types::{Bitcoin, BoundedRatio, Height, Sats, StoredF64, Timestamp};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use vecdb::ReadableVec;

use super::{PrimaryValues, decay::DecayFit};

#[derive(Default)]
pub(crate) struct PrimaryBatch {
    timestamps: Vec<Timestamp>,
    transfer_volumes: AgeRange<Vec<Sats>>,
    coindays_created: AgeRange<Vec<StoredF64>>,
    accounting: CohortAccounting,
}

impl PrimaryBatch {
    pub(crate) fn collect_into(
        &mut self,
        timestamps: &impl ReadableVec<Height, Timestamp>,
        transfer_volumes: &AgeRange<&impl ReadableVec<Height, Sats>>,
        coindays_created: &AgeRange<&impl ReadableVec<Height, StoredF64>>,
        accounting: &AccountingSources<'_>,
        start: usize,
        end: usize,
    ) {
        timestamps.collect_range_into_at(start, end, &mut self.timestamps);
        collect_age_range(transfer_volumes, &mut self.transfer_volumes, start, end);
        collect_age_range(coindays_created, &mut self.coindays_created, start, end);
        accounting.collect_into(start, end, &mut self.accounting);
    }

    #[inline]
    fn len(&self) -> usize {
        self.timestamps.len()
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
        let aggregates = WeightedCohortAggregates::from_fn(|id| {
            let mobility = *id.select(&mobilities);
            self.accounting.weighted(id, offset, mobility)
        });

        PrimaryValues {
            spending_rate: AgeRange::from_fn(|id| StoredF64::from(*id.select(&hazards))),
            spending_exposure: AgeRange::from_fn(|id| StoredF64::from(*id.select(&exposures))),
            mobility: mobilities,
            under_4m: aggregates.under_4m,
            under_6m: aggregates.under_6m,
            over_4m: aggregates.over_4m,
            over_6m: aggregates.over_6m,
            terms: aggregates.terms,
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
    use brk_types::{Cents, CentsSats, CentsSquaredSats};

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
    fn age_threshold_prices_follow_bounded_mobility_and_exact_cutoffs() {
        let bounds = AgeBand::all();
        let batch = PrimaryBatch {
            timestamps: vec![Timestamp::from(20 * 365 * 86_400_u32)],
            transfer_volumes: AgeRange::from_fn(|id| {
                vec![Sats::from(100_000_000_u64 / (id.index() as u64 + 1))]
            }),
            coindays_created: AgeRange::from_fn(|_| vec![StoredF64::from(100.0)]),
            accounting: CohortAccounting {
                supplies: AgeRange::from_fn(|_| vec![Sats::from(100_000_000_u64)]),
                loss_supplies: AgeRange::from_fn(|_| vec![Sats::ZERO]),
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
            },
        };
        let values = batch.primary_values(0, Timestamp::ZERO, &bounds);
        for (days, older, actual) in [
            (120, false, values.under_4m),
            (150, false, values.terms.short),
            (180, false, values.under_6m),
            (120, true, values.over_4m),
            (150, true, values.terms.long),
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
                    id.select(&batch.accounting.supplies)[0],
                    Sats::ZERO,
                    Cents::new((id.index() as u64 + 1) * 100),
                    weight,
                );
                expected.capitalized_price.add(
                    id.select(&batch.accounting.cap_raw)[0],
                    id.select(&batch.accounting.capitalized_cap_raw)[0],
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
