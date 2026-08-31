use brk_error::Result;

use bitview_cohort::{AgeRange, AgeRangeId, ByTerm, TERM_FILTERS, UTXOAggregate};
use bitview_compute::{
    AgeBand, MINIMUM_DURATION_DAYS, WeightedCohortContribution, WeightedCohortState, WeightedRatio,
    db_utils::validate_any_computed_version_or_reset,
};
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_indexer::{Indexer, Lengths};
use brk_exit::Exit;
use brk_types::{Bitcoin, Cents, Height, Sats, StoredF64, Timestamp, Version};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use vecdb::{AnyStoredVec, ColumnId, ReadableVec, VecValue, WritableVec};

use super::Vecs;
use crate::{AGE_COHORT_COUNT, AggregateSources, Dependencies, HorizonId, Horizons};

const WRITE_INTERVAL: usize = 20_000;

#[derive(Clone, Copy)]
struct DecayFit {
    slope: f64,
    tau: f64,
    anchor_age: f64,
    anchor_hazard: f64,
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<Self::Output> {
        self.compute_inner(
            dependencies.indexer,
            dependencies.mappings,
            dependencies.distribution,
            context.exit(),
        )
    }
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
            horizon_supply_in_loss: HorizonId::from_fn(|_| WeightedRatio::default()),
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
        for horizon in HorizonId::ALL {
            let ratio = horizon.select_mut(&mut self.horizon_supply_in_loss);
            let weights = horizon.select(horizon_mobilities);
            ratio.add(loss, total, *age.select(weights));
        }
        contribution
    }

    fn merged(mut self, other: Self) -> Self {
        self.weighted = self.weighted.merged(other.weighted);
        for horizon in HorizonId::ALL {
            horizon
                .select_mut(&mut self.horizon_supply_in_loss)
                .merge(*horizon.select(&other.horizon_supply_in_loss));
        }
        self
    }
}

struct PrimaryBatch {
    timestamps: Vec<Timestamp>,
    transfer_volumes: AgeRange<Vec<Sats>>,
    coindays_created: Vec<AgeRange<StoredF64>>,
    supplies: AgeRange<Vec<Sats>>,
    loss_supplies: AgeRange<Vec<Sats>>,
    realized_caps: AgeRange<Vec<Cents>>,
}

impl PrimaryBatch {
    #[allow(clippy::too_many_arguments)]
    fn collect(
        timestamps: &impl ReadableVec<Height, Timestamp>,
        transfer_volumes: &AgeRange<&impl ReadableVec<Height, Sats>>,
        coindays_created: &impl ReadableVec<Height, AgeRange<StoredF64>>,
        supplies: &AgeRange<&impl ReadableVec<Height, Sats>>,
        loss_supplies: &AgeRange<&impl ReadableVec<Height, Sats>>,
        realized_caps: &AgeRange<&impl ReadableVec<Height, Cents>>,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            timestamps: timestamps.collect_range_at(start, end),
            transfer_volumes: Self::collect_age_range(transfer_volumes, start, end),
            coindays_created: coindays_created.collect_range_at(start, end),
            supplies: Self::collect_age_range(supplies, start, end),
            loss_supplies: Self::collect_age_range(loss_supplies, start, end),
            realized_caps: Self::collect_age_range(realized_caps, start, end),
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

    fn rows(&self, genesis_timestamp: Timestamp, bounds: &AgeRange<AgeBand>) -> Vec<PrimaryRow> {
        (0..self.len())
            .into_par_iter()
            .map(|offset| self.row(offset, genesis_timestamp, bounds))
            .collect()
    }

    fn row(
        &self,
        offset: usize,
        genesis_timestamp: Timestamp,
        bounds: &AgeRange<AgeBand>,
    ) -> PrimaryRow {
        let hazards = AgeRange::from_fn(|id| {
            Self::spending_rate(
                id.select(&self.transfer_volumes)[offset],
                *id.get(&self.coindays_created[offset]),
            )
        });
        let network_age = self.timestamps[offset]
            .difference_in_days_between_float(genesis_timestamp)
            .max(MINIMUM_DURATION_DAYS);
        let exposures = DecayFit::exposures(&hazards, network_age, bounds);
        let mobilities = AgeRange::from_fn(|id| AgeBand::mobility(*id.select(&exposures)));
        let horizon_mobilities: Horizons<AgeRange<f64>> = HorizonId::from_fn(|horizon| {
            let horizon = horizon.days();
            AgeRange::from_fn(|age| AgeBand::horizon_mobility(&hazards, age, horizon, bounds))
        });
        let mut terms = ByTerm::<AggregateState>::default();
        let mut mobile_supply = AgeRange::default();
        let mut immobile_supply = AgeRange::default();

        for &id in AgeRangeId::ALL {
            let mobility = StoredF64::from(*id.select(&mobilities));
            let total_supply = id.select(&self.supplies)[offset];
            let total_cap = id.select(&self.realized_caps)[offset];
            let loss_supply = id.select(&self.loss_supplies)[offset];

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

        PrimaryRow {
            spending_rate: AgeRangeId::from_fn(|id| StoredF64::from(*id.select(&hazards))),
            spending_exposure: AgeRangeId::from_fn(|id| StoredF64::from(*id.select(&exposures))),
            mobile_supply: AgeRangeId::from_fn(|id| *id.select(&mobile_supply)),
            immobile_supply: AgeRangeId::from_fn(|id| *id.select(&immobile_supply)),
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

struct PrimaryRow {
    spending_rate: AgeRange<StoredF64>,
    spending_exposure: AgeRange<StoredF64>,
    mobile_supply: AgeRange<Sats>,
    immobile_supply: AgeRange<Sats>,
    terms: ByTerm<AggregateState>,
}

impl Vecs {
    fn compute_inner(
        &mut self,
        indexer: &Indexer,
        mappings: &bitview_plugin_mappings::Vecs,
        distribution: &bitview_plugin_distribution::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        let starting_lengths = indexer.safe_lengths();
        let transfer_volumes = AgeRange::from_fn(|id| {
            &id.select(
                &distribution
                    .cohorts
                    .activity
                    .transfer_volume
                    .cohorts
                    .age
                    .range,
            )
            .cumulative
            .sats
            .height
        });
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
        let coindays_created = &distribution.coindays_created.cumulative;

        self.compute_primary(
            &starting_lengths,
            &mappings.timestamp.monotonic,
            &transfer_volumes,
            coindays_created,
            &supplies,
            &loss_supplies,
            &realized_caps,
            exit,
        )?;

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_primary(
        &mut self,
        starting_lengths: &Lengths,
        timestamps: &impl ReadableVec<Height, Timestamp>,
        transfer_volumes: &AgeRange<&impl ReadableVec<Height, Sats>>,
        coindays_created: &impl ReadableVec<Height, AgeRange<StoredF64>>,
        supplies: &AgeRange<&impl ReadableVec<Height, Sats>>,
        loss_supplies: &AgeRange<&impl ReadableVec<Height, Sats>>,
        realized_caps: &AgeRange<&impl ReadableVec<Height, Cents>>,
        exit: &Exit,
    ) -> Result<Height> {
        let source_version = Version::combine_all(
            std::iter::once(timestamps.version())
                .chain(transfer_volumes.iter().map(|vec| vec.version()))
                .chain(std::iter::once(coindays_created.version()))
                .chain(supplies.iter().map(|vec| vec.version()))
                .chain(loss_supplies.iter().map(|vec| vec.version()))
                .chain(realized_caps.iter().map(|vec| vec.version())),
        );

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
        let bounds = AgeBand::all();
        let mut chunk_start = start;
        while chunk_start < source_end {
            let chunk_end = (chunk_start + WRITE_INTERVAL).min(source_end);
            let batch = PrimaryBatch::collect(
                timestamps,
                transfer_volumes,
                coindays_created,
                supplies,
                loss_supplies,
                realized_caps,
                chunk_start,
                chunk_end,
            );
            for row in batch.rows(genesis_timestamp, &bounds) {
                self.push_primary(row);
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

    fn push_primary(&mut self, row: PrimaryRow) {
        self.age_range.spending_rate.push(row.spending_rate);
        self.age_range.spending_exposure.push(row.spending_exposure);
        self.age_range.supply.mobile.push(row.mobile_supply);
        self.age_range.supply.immobile.push(row.immobile_supply);

        let all = row.terms.short.merged(row.terms.long);
        self.aggregate_sources.push(row.terms, all);
    }

    fn primary_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        [
            self.age_range.spending_rate.stored_mut(),
            self.age_range.spending_exposure.stored_mut(),
            self.age_range.supply.mobile.stored_mut(),
            self.age_range.supply.immobile.stored_mut(),
        ]
        .into_iter()
        .chain(self.aggregate_sources.primary_vecs_mut())
    }
}

impl AggregateSources {
    fn push(&mut self, terms: ByTerm<AggregateState>, all: AggregateState) {
        self.supply.mobile.push(ByTerm {
            short: terms.short.weighted.weighted_supply,
            long: terms.long.weighted.weighted_supply,
        });
        self.supply.immobile.push(ByTerm {
            short: terms.short.weighted.complement_supply,
            long: terms.long.weighted.complement_supply,
        });
        self.supply_in_loss_share.push(UTXOAggregate {
            all: all.weighted.supply_in_loss.value(),
            sth: terms.short.weighted.supply_in_loss.value(),
            lth: terms.long.weighted.supply_in_loss.value(),
        });
        for horizon in HorizonId::ALL {
            horizon.select_mut(&mut self.horizon).push(UTXOAggregate {
                all: horizon.select(&all.horizon_supply_in_loss).value(),
                sth: horizon.select(&terms.short.horizon_supply_in_loss).value(),
                lth: horizon.select(&terms.long.horizon_supply_in_loss).value(),
            });
        }
        self.cap.push(ByTerm {
            short: terms.short.weighted.weighted_cap,
            long: terms.long.weighted.weighted_cap,
        });
        self.price.push(UTXOAggregate {
            all: all.weighted.realized_price(),
            sth: terms.short.weighted.realized_price(),
            lth: terms.long.weighted.realized_price(),
        });
    }

    fn primary_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        let Self {
            supply,
            supply_in_loss_share,
            horizon,
            cap,
            price,
        } = self;
        let Horizons {
            _8y,
            _4y,
            _2y,
            _1y,
            _6m,
            _3m,
            _1m,
        } = horizon;
        [
            &mut supply.mobile as &mut dyn AnyStoredVec,
            &mut supply.immobile,
            supply_in_loss_share,
            cap,
            price,
        ]
        .into_iter()
        .chain(
            [_8y, _4y, _2y, _1y, _6m, _3m, _1m]
                .into_iter()
                .map(|horizon| horizon as &mut dyn AnyStoredVec),
        )
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
        assert_eq!(AgeBand::mobility(0.0), 0.0);
        assert!((AgeBand::mobility(2.0_f64.ln()) - 0.5).abs() < 1e-12);
        assert!((AgeBand::mobility(1e-15) - 1e-15).abs() < 1e-27);
        assert!(AgeBand::mobility(1_000.0) < 1.0);
        assert_eq!(AgeBand::mobility(f64::INFINITY), 1.0 - 1e-12);
        assert_eq!(AgeBand::mobility(f64::NAN), 0.0);
    }

    #[test]
    fn fixed_horizon_compounds_hazards_across_age_ranges() {
        let bounds = AgeBand::all();
        let hazards = AgeRange::from_fn(|_| 0.01);
        let probability =
            AgeBand::horizon_mobility(&hazards, AgeRangeId::From1DTo1W, 30.0, &bounds);

        assert!((probability - AgeBand::mobility(0.3)).abs() < 1e-12);
    }

    #[test]
    fn decay_fit_recovers_an_exponential_lifetime() {
        let bounds = AgeBand::all();
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
        let bounds = AgeBand::all();
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
        let horizons = HorizonId::from_fn(|_| AgeRange::from_fn(|_| 0.25));
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
        for horizon in HorizonId::ALL {
            assert_eq!(
                horizon.select(&merged.horizon_supply_in_loss).value(),
                horizon.select(&direct.horizon_supply_in_loss).value(),
            );
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
