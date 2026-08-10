use brk_error::Result;
use brk_indexer::{Indexer, Lengths};
use brk_types::{Bitcoin, Cents, Height, Sats, StoredF64, Timestamp, Version};
use vecdb::{AnyStoredVec, ColumnId, Exit, ReadableVec, WritableVec};

use brk_cohort::{AgeRange, AgeRangeId, ByTerm, TERM_FILTERS};

use super::super::cointime;
use super::{
    AGE_COHORT_COUNT, AgeBand, AggregateVecs, Horizons, MINIMUM_DURATION_DAYS, Vecs,
    age_bounds_days, horizon_mobility, mobility,
};
use crate::{
    distribution,
    frameworks::{WeightedCohortContribution, WeightedCohortState, WeightedRatio},
    indexes,
    internal::db_utils::validate_any_computed_version_or_reset,
    price,
};

const WRITE_INTERVAL: usize = 10_000;

#[derive(Clone, Copy)]
struct DecayFit {
    slope: f64,
    tau: f64,
    anchor_age: f64,
    anchor_hazard: f64,
}

#[derive(Clone, Copy)]
struct AggregateState {
    weighted: WeightedCohortState,
    horizon_supply_in_loss: Horizons<WeightedRatio>,
}

impl Default for AggregateState {
    fn default() -> Self {
        Self {
            weighted: WeightedCohortState::default(),
            horizon_supply_in_loss: Horizons::from_fn(|_, _| WeightedRatio::default()),
        }
    }
}

impl AggregateState {
    fn add(
        &mut self,
        total_supply: Sats,
        loss_supply: Sats,
        total_cap: Cents,
        mobility: StoredF64,
        horizon_mobilities: &Horizons<AgeRange<f64>>,
        age: AgeRangeId,
    ) -> WeightedCohortContribution {
        let contribution = self
            .weighted
            .add(total_supply, loss_supply, total_cap, mobility);
        let total = total_supply.as_u128() as f64;
        let loss = loss_supply.as_u128() as f64;
        for (ratio, weights) in self
            .horizon_supply_in_loss
            .iter_mut()
            .zip(horizon_mobilities.iter())
        {
            ratio.add(loss, total, *age.select(weights));
        }
        contribution
    }

    fn merged(mut self, other: Self) -> Self {
        self.weighted = self.weighted.merged(other.weighted);
        for (ratio, other) in self
            .horizon_supply_in_loss
            .iter_mut()
            .zip(other.horizon_supply_in_loss.iter())
        {
            ratio.merge(*other);
        }
        self
    }
}

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        indexes: &indexes::Vecs,
        _prices: &price::Vecs,
        distribution: &distribution::Vecs,
        cointime: &cointime::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();
        let transfer_volumes = AgeRange::from_fn(|id| {
            &id.select(&distribution.utxo_cohorts.age_range)
                .metrics
                .activity
                .transfer_volume
                .cumulative
                .sats
                .height
        });
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
        let coindays_created = &cointime.age_range.coindays_created.cumulative;

        self.compute_primary(
            &starting_lengths,
            &indexes.timestamp.monotonic,
            &transfer_volumes,
            coindays_created,
            &supplies,
            &loss_supplies,
            &realized_caps,
            exit,
        )?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_primary<T, D, S, C>(
        &mut self,
        starting_lengths: &Lengths,
        timestamps: &T,
        transfer_volumes: &AgeRange<&S>,
        coindays_created: &D,
        supplies: &AgeRange<&S>,
        loss_supplies: &AgeRange<&S>,
        realized_caps: &AgeRange<&C>,
        exit: &Exit,
    ) -> Result<Height>
    where
        T: ReadableVec<Height, Timestamp>,
        D: ReadableVec<Height, [StoredF64; AGE_COHORT_COUNT]>,
        S: ReadableVec<Height, Sats>,
        C: ReadableVec<Height, Cents>,
    {
        let source_version: Version = std::iter::once(timestamps.version())
            .chain(transfer_volumes.iter().map(|vec| vec.version()))
            .chain(std::iter::once(coindays_created.version()))
            .chain(supplies.iter().map(|vec| vec.version()))
            .chain(loss_supplies.iter().map(|vec| vec.version()))
            .chain(realized_caps.iter().map(|vec| vec.version()))
            .sum();

        for vec in self.primary_vecs_mut() {
            validate_any_computed_version_or_reset(vec, source_version)?;
        }

        let start = self
            .primary_vecs_mut()
            .map(|vec| vec.len())
            .min()
            .unwrap_or_default()
            .min(usize::from(starting_lengths.height));

        for vec in self.primary_vecs_mut() {
            vec.any_truncate_if_needed_at(start)?;
        }

        let source_end = transfer_volumes
            .iter()
            .map(|vec| vec.len())
            .chain(std::iter::once(coindays_created.len()))
            .chain(supplies.iter().map(|vec| vec.len()))
            .chain(loss_supplies.iter().map(|vec| vec.len()))
            .chain(realized_caps.iter().map(|vec| vec.len()))
            .chain(std::iter::once(timestamps.len()))
            .min()
            .unwrap_or_default();

        if source_end == 0 {
            return Ok(Height::ZERO);
        }

        let genesis_timestamp = timestamps
            .collect_one(Height::ZERO)
            .unwrap_or(Timestamp::ZERO);
        let bounds = age_bounds_days();
        let mut chunk_start = start;
        while chunk_start < source_end {
            let chunk_end = (chunk_start + WRITE_INTERVAL).min(source_end);
            let timestamp_batch = timestamps.collect_range_at(chunk_start, chunk_end);
            let transfer_batches = AgeRange::from_fn(|id| {
                id.select(transfer_volumes)
                    .collect_range_at(chunk_start, chunk_end)
            });
            let coinday_batch = coindays_created.collect_range_at(chunk_start, chunk_end);
            let supply_batches = AgeRange::from_fn(|id| {
                id.select(supplies).collect_range_at(chunk_start, chunk_end)
            });
            let loss_supply_batches = AgeRange::from_fn(|id| {
                id.select(loss_supplies)
                    .collect_range_at(chunk_start, chunk_end)
            });
            let cap_batches = AgeRange::from_fn(|id| {
                id.select(realized_caps)
                    .collect_range_at(chunk_start, chunk_end)
            });

            for offset in 0..(chunk_end - chunk_start) {
                let hazards = AgeRange::from_fn(|id| {
                    spending_rate(
                        id.select(&transfer_batches)[offset],
                        *id.get(&coinday_batch[offset]),
                    )
                });
                let network_age = timestamp_batch[offset]
                    .difference_in_days_between_float(genesis_timestamp)
                    .max(MINIMUM_DURATION_DAYS);
                let exposures = DecayFit::exposures(&hazards, network_age, &bounds);
                let mobilities = AgeRange::from_fn(|id| mobility(*id.select(&exposures)));
                let horizon_mobilities: Horizons<AgeRange<f64>> =
                    Horizons::from_fn(|_, horizon| {
                        AgeRange::from_fn(|age| horizon_mobility(&hazards, age, horizon, &bounds))
                    });
                self.age_range.spending_rate.push(AgeRangeId::from_fn(|id| {
                    StoredF64::from(*id.select(&hazards))
                }));
                self.age_range
                    .spending_exposure
                    .push(AgeRangeId::from_fn(|id| {
                        StoredF64::from(*id.select(&exposures))
                    }));

                let mut terms = ByTerm::<AggregateState>::default();
                let mut mobile_supply = AgeRange::default();
                let mut immobile_supply = AgeRange::default();

                for &id in AgeRangeId::ALL {
                    let mobility = StoredF64::from(*id.select(&mobilities));
                    let total_supply = id.select(&supply_batches)[offset];
                    let total_cap = id.select(&cap_batches)[offset];
                    let loss_supply = id.select(&loss_supply_batches)[offset];

                    let term = if TERM_FILTERS.short.includes(id.filter()) {
                        &mut terms.short
                    } else {
                        &mut terms.long
                    };
                    let contribution = term.add(
                        total_supply,
                        loss_supply,
                        total_cap,
                        mobility,
                        &horizon_mobilities,
                        id,
                    );

                    *id.select_mut(&mut mobile_supply) = contribution.weighted_supply;
                    *id.select_mut(&mut immobile_supply) = contribution.complement_supply;
                }

                self.age_range
                    .supply
                    .mobile
                    .push(AgeRangeId::from_fn(|id| *id.select(&mobile_supply)));
                self.age_range
                    .supply
                    .immobile
                    .push(AgeRangeId::from_fn(|id| *id.select(&immobile_supply)));

                self.all.push(terms.short.merged(terms.long));
                self.sth.push(terms.short);
                self.lth.push(terms.long);
            }

            {
                let _lock = exit.lock();
                for vec in self.primary_vecs_mut() {
                    vec.write()?;
                }
            }
            chunk_start = chunk_end;
        }

        Ok(Height::from(start))
    }

    fn primary_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        [
            self.age_range.spending_rate.stored_mut(),
            self.age_range.spending_exposure.stored_mut(),
            self.age_range.supply.mobile.stored_mut(),
            self.age_range.supply.immobile.stored_mut(),
        ]
        .into_iter()
        .chain(self.all.primary_vecs_mut())
        .chain(self.sth.primary_vecs_mut())
        .chain(self.lth.primary_vecs_mut())
    }
}

impl AggregateVecs {
    fn push(&mut self, state: AggregateState) {
        self.supply
            .mobile
            .sats
            .height
            .push(state.weighted.weighted_supply);
        self.supply
            .immobile
            .sats
            .height
            .push(state.weighted.complement_supply);
        self.supply_in_loss_share
            .height
            .push(state.weighted.supply_in_loss.value());
        for (output, ratio) in self
            .horizon
            .iter_mut()
            .zip(state.horizon_supply_in_loss.iter())
        {
            output.supply_in_loss_share.height.push(ratio.value());
        }
        self.cap.cents.height.push(state.weighted.weighted_cap);
        self.price
            .cents
            .height
            .push(state.weighted.realized_price());
    }

    fn primary_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        [
            &mut self.supply.mobile.sats.height as &mut dyn AnyStoredVec,
            &mut self.supply.immobile.sats.height,
            &mut self.supply_in_loss_share.height,
            &mut self.cap.cents.height,
            &mut self.price.cents.height,
        ]
        .into_iter()
        .chain(
            self.horizon
                .iter_mut()
                .map(|horizon| &mut horizon.supply_in_loss_share.height as &mut dyn AnyStoredVec),
        )
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

impl DecayFit {
    fn fit(hazards: &AgeRange<f64>, network_age: f64, bounds: &AgeRange<AgeBand>) -> Option<Self> {
        let mut total_duration = 0.0;
        let mut weighted_age = 0.0;
        let mut weighted_log_hazard = 0.0;
        let mut anchor = None;

        for &id in &AgeRangeId::ALL[..AGE_COHORT_COUNT - 1] {
            let band = *id.select(bounds);
            let hazard = *id.select(hazards);
            if band.upper > network_age || !hazard.is_finite() || hazard <= 0.0 {
                continue;
            }

            let age = (band.lower + band.upper) / 2.0;
            let duration = band.upper - band.lower;
            let log_hazard = hazard.ln();
            total_duration += duration;
            weighted_age += duration * age;
            weighted_log_hazard += duration * log_hazard;
            anchor = Some((band.upper, hazard));
        }

        if total_duration <= 0.0 {
            return None;
        }

        let mean_age = weighted_age / total_duration;
        let mean_log_hazard = weighted_log_hazard / total_duration;
        let mut covariance = 0.0;
        let mut age_variance = 0.0;

        for &id in &AgeRangeId::ALL[..AGE_COHORT_COUNT - 1] {
            let band = *id.select(bounds);
            let hazard = *id.select(hazards);
            if band.upper > network_age || !hazard.is_finite() || hazard <= 0.0 {
                continue;
            }

            let age = (band.lower + band.upper) / 2.0;
            let duration = band.upper - band.lower;
            let log_hazard = hazard.ln();
            let age_offset = age - mean_age;
            covariance += duration * age_offset * (log_hazard - mean_log_hazard);
            age_variance += duration * age_offset.powi(2);
        }

        if age_variance <= f64::EPSILON {
            return None;
        }

        let slope = covariance / age_variance;
        if slope >= 0.0 {
            return None;
        }

        let tau = -1.0 / slope;
        if !tau.is_finite() || tau <= 0.0 {
            return None;
        }

        let (anchor_age, anchor_hazard) = anchor?;
        Some(Self {
            slope,
            tau,
            anchor_age,
            anchor_hazard,
        })
    }

    fn exposures(
        hazards: &AgeRange<f64>,
        network_age: f64,
        bounds: &AgeRange<AgeBand>,
    ) -> AgeRange<f64> {
        let Some(fit) = Self::fit(hazards, network_age, bounds) else {
            return AgeRange::default();
        };

        AgeRange::from_fn(|start_band| fit.exposure(hazards, start_band, network_age, bounds))
    }

    fn exposure(
        self,
        hazards: &AgeRange<f64>,
        start_band: AgeRangeId,
        network_age: f64,
        bounds: &AgeRange<AgeBand>,
    ) -> f64 {
        let start = *start_band.select(bounds);
        let occupied_upper = if start.upper.is_finite() {
            start.upper.min(network_age.max(start.lower))
        } else {
            start.lower
        };
        let mut age = if start.upper.is_finite() {
            (start.lower + occupied_upper) / 2.0
        } else {
            start.lower
        };
        let mut exposure = 0.0;

        for &band_id in &AgeRangeId::ALL[start_band.index()..AGE_COHORT_COUNT - 1] {
            let band = *band_id.select(bounds);
            let duration = (band.upper - age.max(band.lower)).max(MINIMUM_DURATION_DAYS);
            let hazard = *band_id.select(hazards);
            let observed = band.upper <= network_age && hazard.is_finite() && hazard > 0.0;
            if !observed {
                break;
            }

            exposure += hazard * duration;
            age = band.upper;
        }

        let tail = bounds.over_15y;
        let tail_hazard = hazards.over_15y;
        let observed_tail =
            network_age > tail.lower && tail_hazard.is_finite() && tail_hazard > 0.0;
        let (anchor_age, anchor_hazard) = if observed_tail {
            (tail.lower, tail_hazard)
        } else {
            (self.anchor_age, self.anchor_hazard)
        };
        let continuation_age = age.max(anchor_age);
        let continuation_hazard =
            anchor_hazard * (self.slope * (continuation_age - anchor_age)).exp();
        exposure + (continuation_hazard * self.tau).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobility_is_the_complement_of_survival() {
        assert_eq!(mobility(0.0), 0.0);
        assert!((mobility(2.0_f64.ln()) - 0.5).abs() < 1e-12);
        assert!((mobility(1e-15) - 1e-15).abs() < 1e-27);
        assert!(mobility(1_000.0) < 1.0);
        assert_eq!(mobility(f64::INFINITY), 1.0 - 1e-12);
        assert_eq!(mobility(f64::NAN), 0.0);
    }

    #[test]
    fn fixed_horizon_compounds_hazards_across_age_ranges() {
        let bounds = age_bounds_days();
        let hazards = AgeRange::from_fn(|_| 0.01);
        let probability = horizon_mobility(&hazards, AgeRangeId::From1DTo1W, 30.0, &bounds);

        assert!((probability - mobility(0.3)).abs() < 1e-12);
    }

    #[test]
    fn decay_fit_recovers_an_exponential_lifetime() {
        let bounds = age_bounds_days();
        let expected_tau = 1_000.0;
        let hazards = AgeRange::from_fn(|id| {
            let band = *id.select(&bounds);
            let age = if band.upper.is_finite() {
                (band.lower + band.upper) / 2.0
            } else {
                band.lower
            };
            (-age / expected_tau).exp()
        });

        let fit = DecayFit::fit(&hazards, 20.0 * 365.0, &bounds).unwrap();

        assert!((fit.tau - expected_tau).abs() < 1e-9);
    }

    #[test]
    fn oldest_cohort_exposure_is_its_observed_tail_lifetime() {
        let bounds = age_bounds_days();
        let hazards = AgeRange::from_fn(|id| {
            let band = *id.select(&bounds);
            let age = if band.upper.is_finite() {
                (band.lower + band.upper) / 2.0
            } else {
                band.lower
            };
            (-age / 1_000.0).exp()
        });
        let network_age = 20.0 * 365.0;
        let fit = DecayFit::fit(&hazards, network_age, &bounds).unwrap();
        let exposures = DecayFit::exposures(&hazards, network_age, &bounds);

        assert!((exposures.over_15y - hazards.over_15y * fit.tau).abs() < 1e-12);
    }

    #[test]
    fn mobile_and_immobile_supply_are_independently_floored() {
        let total_supply = Sats::from(123_456_789_u64);
        let total_cap = Cents::from(987_654_321_u64);
        let mobility = StoredF64::from(0.321);
        let mut state = WeightedCohortState::default();

        state.add(total_supply, Sats::ZERO, total_cap, mobility);

        let sum = state.weighted_supply + state.complement_supply;
        assert!(sum <= total_supply);
        assert!(total_supply - sum <= Sats::from(1_u64));
        assert!(state.weighted_cap <= total_cap);
    }

    #[test]
    fn term_aggregates_merge_into_all() {
        let horizons = Horizons::from_fn(|_, _| AgeRange::from_fn(|_| 0.25));
        let mut direct = AggregateState::default();
        direct.add(
            Sats::from(100_u64),
            Sats::from(20_u64),
            Cents::from(1_000_u64),
            StoredF64::from(0.3),
            &horizons,
            AgeRangeId::Under1H,
        );
        direct.add(
            Sats::from(200_u64),
            Sats::from(50_u64),
            Cents::from(3_000_u64),
            StoredF64::from(0.4),
            &horizons,
            AgeRangeId::From1HTo1D,
        );

        let mut sth = AggregateState::default();
        sth.add(
            Sats::from(100_u64),
            Sats::from(20_u64),
            Cents::from(1_000_u64),
            StoredF64::from(0.3),
            &horizons,
            AgeRangeId::Under1H,
        );
        let mut lth = AggregateState::default();
        lth.add(
            Sats::from(200_u64),
            Sats::from(50_u64),
            Cents::from(3_000_u64),
            StoredF64::from(0.4),
            &horizons,
            AgeRangeId::From1HTo1D,
        );
        let merged = sth.merged(lth);

        assert_eq!(
            merged.weighted.weighted_supply,
            direct.weighted.weighted_supply
        );
        assert_eq!(
            merged.weighted.complement_supply,
            direct.weighted.complement_supply
        );
        assert_eq!(merged.weighted.weighted_cap, direct.weighted.weighted_cap);
        assert_eq!(
            merged.weighted.supply_in_loss.value(),
            direct.weighted.supply_in_loss.value()
        );
        for (merged, direct) in merged
            .horizon_supply_in_loss
            .iter()
            .zip(direct.horizon_supply_in_loss.iter())
        {
            assert_eq!(merged.value(), direct.value());
        }
    }

    #[test]
    fn age_ranges_belong_to_exactly_one_term() {
        for id in AgeRangeId::ALL {
            assert_ne!(
                TERM_FILTERS.short.includes(id.filter()),
                TERM_FILTERS.long.includes(id.filter())
            );
        }
    }
}
