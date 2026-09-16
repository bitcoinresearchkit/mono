use std::iter;

use bitview_cohort::{AgeRange, AgeRangeId, ByTerm, Term, UTXOAggregateId};
use bitview_compute::{
    AgeBand, MINIMUM_DURATION_DAYS, WeightedCohortContribution, WeightedCohortState, WeightedRatio,
};
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_indexer::Lengths;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{
    Bitcoin, BoundedRatio, Cents, CentsSats, CentsSquaredSats, Height, Sats, StoredF64, Timestamp,
    Version,
};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use vecdb::{AnyStoredVec, ReadableVec, VecValue, WritableVec};

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
        let Dependencies {
            indexer,
            mappings,
            distribution,
        } = dependencies;
        let exit = context.exit();

        self.db.sync_bg_tasks()?;

        let starting_lengths = indexer.safe_lengths();
        let transfer_volumes = AgeRange::from_fn(|id| {
            &id.select(
                &distribution
                    .cohorts
                    .activity
                    .transfer_volume
                    .cohorts
                    .utxo
                    .age,
            )
            .cumulative
            .sats
            .height
        });
        let supplies = AgeRange::from_fn(|id| {
            &id.select(&distribution.cohorts.supply.total.cohorts.utxo.age)
                .sats
                .height
        });
        let loss_supplies = AgeRange::from_fn(|id| {
            &id.select(&distribution.cohorts.supply.in_loss.cohorts.age)
                .sats
                .height
        });
        let realized_caps = AgeRange::from_fn(|id| {
            &id.select(&distribution.cohorts.realized.cap.cohorts.utxo.age)
                .cents
                .height
        });
        let cap_raw = AgeRange::from_fn(|id| id.select(&distribution.cohorts.realized.cap_raw.age));
        let capitalized_cap_raw = AgeRange::from_fn(|id| {
            id.select(&distribution.cohorts.realized.capitalized_cap_raw.age)
        });
        let coindays_created =
            AgeRange::from_fn(|id| &id.select(&distribution.coindays_created).cumulative.height);

        self.compute_primary(
            &starting_lengths,
            &mappings.timestamp.monotonic,
            &transfer_volumes,
            &coindays_created,
            &supplies,
            &loss_supplies,
            &realized_caps,
            &cap_raw,
            &capitalized_cap_raw,
            exit,
        )?;

        context.compact_database(&self.db);

        Ok(())
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
        mobility: BoundedRatio,
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
    coindays_created: AgeRange<Vec<StoredF64>>,
    supplies: AgeRange<Vec<Sats>>,
    loss_supplies: AgeRange<Vec<Sats>>,
    realized_caps: AgeRange<Vec<Cents>>,
    cap_raw: AgeRange<Vec<CentsSats>>,
    capitalized_cap_raw: AgeRange<Vec<CentsSquaredSats>>,
}

impl PrimaryBatch {
    #[allow(clippy::too_many_arguments)]
    fn collect(
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

    fn primary_values_batch(
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
        let horizon_mobilities: Horizons<AgeRange<f64>> = HorizonId::from_fn(|horizon| {
            let horizon = horizon.days();
            AgeRange::from_fn(|age| AgeBand::horizon_mobility(&hazards, age, horizon, bounds))
        });
        let mut terms = ByTerm::<AggregateState>::default();
        let mut under_4m = WeightedCohortState::default();
        let mut under_6m = WeightedCohortState::default();
        let mut over_4m = WeightedCohortState::default();
        let mut over_6m = WeightedCohortState::default();

        for &id in AgeRangeId::ALL {
            let mobility = *id.select(&mobilities);
            let total_supply = id.select(&self.supplies)[offset];
            let total_cap = id.select(&self.realized_caps)[offset];
            let loss_supply = id.select(&self.loss_supplies)[offset];

            let term = if id.term() == Term::Sth {
                &mut terms.short
            } else {
                &mut terms.long
            };
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
            *term = term.merged(contribution);
            if id >= AgeRangeId::From4MTo5M {
                over_4m = over_4m.merged(contribution.weighted);
            }
            if id >= AgeRangeId::From6MTo9M {
                over_6m = over_6m.merged(contribution.weighted);
            }
            // AgeRangeId::ALL is ordered youngest to oldest.
            if id == AgeRangeId::From3MTo4M {
                under_4m = terms.short.weighted;
            } else if id == AgeRangeId::From5MTo6M {
                under_6m = terms.short.weighted.merged(terms.long.weighted);
            }
        }

        PrimaryValues {
            spending_rate: AgeRange::from_fn(|id| StoredF64::from(*id.select(&hazards))),
            spending_exposure: AgeRange::from_fn(|id| StoredF64::from(*id.select(&exposures))),
            mobility: mobilities,
            under_4m,
            under_6m,
            over_4m,
            over_6m,
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

struct PrimaryValues {
    spending_rate: AgeRange<StoredF64>,
    spending_exposure: AgeRange<StoredF64>,
    mobility: AgeRange<BoundedRatio>,
    terms: ByTerm<AggregateState>,
    under_4m: WeightedCohortState,
    under_6m: WeightedCohortState,
    over_4m: WeightedCohortState,
    over_6m: WeightedCohortState,
}

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    fn compute_primary(
        &mut self,
        starting_lengths: &Lengths,
        timestamps: &impl ReadableVec<Height, Timestamp>,
        transfer_volumes: &AgeRange<&impl ReadableVec<Height, Sats>>,
        coindays_created: &AgeRange<&impl ReadableVec<Height, StoredF64>>,
        supplies: &AgeRange<&impl ReadableVec<Height, Sats>>,
        loss_supplies: &AgeRange<&impl ReadableVec<Height, Sats>>,
        realized_caps: &AgeRange<&impl ReadableVec<Height, Cents>>,
        cap_raw: &AgeRange<&impl ReadableVec<Height, CentsSats>>,
        capitalized_cap_raw: &AgeRange<&impl ReadableVec<Height, CentsSquaredSats>>,
        exit: &Exit,
    ) -> Result<Height> {
        let source_version = Version::combine_all(
            iter::once(timestamps.version())
                .chain(transfer_volumes.iter().map(|vec| vec.version()))
                .chain(coindays_created.iter().map(|vec| vec.version()))
                .chain(supplies.iter().map(|vec| vec.version()))
                .chain(loss_supplies.iter().map(|vec| vec.version()))
                .chain(realized_caps.iter().map(|vec| vec.version()))
                .chain(cap_raw.iter().map(|vec| vec.version()))
                .chain(capitalized_cap_raw.iter().map(|vec| vec.version())),
        );

        for vec in self.primary_vecs_mut() {
            vec.any_validate_computed_version_or_reset(source_version)?;
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
            .chain(coindays_created.iter().map(|vec| vec.len()))
            .chain(supplies.iter().map(|vec| vec.len()))
            .chain(loss_supplies.iter().map(|vec| vec.len()))
            .chain(realized_caps.iter().map(|vec| vec.len()))
            .chain(cap_raw.iter().map(|vec| vec.len()))
            .chain(capitalized_cap_raw.iter().map(|vec| vec.len()))
            .chain(iter::once(timestamps.len()))
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
                cap_raw,
                capitalized_cap_raw,
                chunk_start,
                chunk_end,
            );
            for values in batch.primary_values_batch(genesis_timestamp, &bounds) {
                self.push_primary(values);
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

    fn push_primary(&mut self, values: PrimaryValues) {
        for (target, value) in self
            .age_range
            .spending_rate
            .iter_mut()
            .zip(values.spending_rate.iter())
        {
            target.height.push(*value);
        }
        for (target, value) in self
            .age_range
            .spending_exposure
            .age_range
            .iter_mut()
            .zip(values.spending_exposure.iter())
        {
            target.height.push(*value);
        }
        for (target, value) in self
            .age_range
            .mobility_source
            .iter_mut()
            .zip(values.mobility.iter())
        {
            target.push(*value);
        }

        self.aggregate_sources
            .under_4m_price
            .push(values.under_4m.realized_price());
        self.aggregate_sources
            .under_4m_capitalized_price
            .push(values.under_4m.capitalized_price.value());
        self.aggregate_sources
            .under_6m_price
            .push(values.under_6m.realized_price());
        self.aggregate_sources
            .under_6m_capitalized_price
            .push(values.under_6m.capitalized_price.value());
        self.aggregate_sources
            .over_4m_price
            .push(values.over_4m.realized_price());
        self.aggregate_sources
            .over_4m_capitalized_price
            .push(values.over_4m.capitalized_price.value());
        self.aggregate_sources
            .over_6m_price
            .push(values.over_6m.realized_price());
        self.aggregate_sources
            .over_6m_capitalized_price
            .push(values.over_6m.capitalized_price.value());
        let all = values.terms.short.merged(values.terms.long);
        self.aggregate_sources.push(values.terms, all);
    }

    fn primary_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.age_range
            .spending_rate
            .iter_mut()
            .map(|v| &mut v.height as &mut dyn AnyStoredVec)
            .chain(
                self.age_range
                    .spending_exposure
                    .age_range
                    .iter_mut()
                    .map(|v| &mut v.height as &mut dyn AnyStoredVec),
            )
            .chain(
                self.age_range
                    .mobility_source
                    .iter_mut()
                    .map(|v| v as &mut dyn AnyStoredVec),
            )
            .chain(self.aggregate_sources.primary_vecs_mut())
    }
}

impl AggregateSources {
    fn push(&mut self, terms: ByTerm<AggregateState>, all: AggregateState) {
        for (id, state) in [
            (UTXOAggregateId::All, all),
            (UTXOAggregateId::Sth, terms.short),
            (UTXOAggregateId::Lth, terms.long),
        ] {
            id.select_mut(&mut self.supply.mobile)
                .push(state.weighted.weighted_supply);
            id.select_mut(&mut self.supply.immobile)
                .push(state.weighted.complement_supply);
            id.select_mut(&mut self.supply_in_loss_share)
                .push(state.weighted.supply_in_loss.value());
            id.select_mut(&mut self.cap)
                .push(state.weighted.weighted_cap);
            id.select_mut(&mut self.capitalized_price)
                .push(state.weighted.capitalized_price.value());
            id.select_mut(&mut self.price)
                .push(state.weighted.realized_price());
            for horizon in HorizonId::ALL {
                id.select_mut(horizon.select_mut(&mut self.horizon))
                    .push(horizon.select(&state.horizon_supply_in_loss).value());
            }
        }
    }

    fn primary_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        let Self {
            supply,
            supply_in_loss_share,
            horizon,
            cap,
            price,
            capitalized_price,
            under_4m_price,
            under_4m_capitalized_price,
            under_6m_price,
            under_6m_capitalized_price,
            over_4m_price,
            over_4m_capitalized_price,
            over_6m_price,
            over_6m_capitalized_price,
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
            &mut supply.mobile.all as &mut dyn AnyStoredVec,
            &mut supply.mobile.sth,
            &mut supply.mobile.lth,
            &mut supply.immobile.all,
            &mut supply.immobile.sth,
            &mut supply.immobile.lth,
            &mut supply_in_loss_share.all,
            &mut supply_in_loss_share.sth,
            &mut supply_in_loss_share.lth,
            &mut cap.all,
            &mut cap.sth,
            &mut cap.lth,
            &mut price.all,
            &mut price.sth,
            &mut price.lth,
            &mut capitalized_price.all,
            &mut capitalized_price.sth,
            &mut capitalized_price.lth,
            under_4m_price,
            under_4m_capitalized_price,
            under_6m_price,
            under_6m_capitalized_price,
            over_4m_price,
            over_4m_capitalized_price,
            over_6m_price,
            over_6m_capitalized_price,
        ]
        .into_iter()
        .chain(
            [_8y, _4y, _2y, _1y, _6m, _3m, _1m]
                .into_iter()
                .flat_map(|h| [&mut h.all as &mut dyn AnyStoredVec, &mut h.sth, &mut h.lth]),
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

    #[test]
    fn term_aggregates_merge_into_all() {
        let horizons = HorizonId::from_fn(|_| AgeRange::from_fn(|_| 0.25));
        let mut direct = AggregateState::default();
        direct.add(
            Sats::from(100_u64),
            Sats::from(20_u64),
            Cents::from(1_000_u64),
            BoundedRatio::from(0.3),
            &horizons,
            AgeRangeId::Under1H,
        );
        direct.add(
            Sats::from(200_u64),
            Sats::from(50_u64),
            Cents::from(3_000_u64),
            BoundedRatio::from(0.4),
            &horizons,
            AgeRangeId::From1HTo1D,
        );

        let mut sth = AggregateState::default();
        sth.add(
            Sats::from(100_u64),
            Sats::from(20_u64),
            Cents::from(1_000_u64),
            BoundedRatio::from(0.3),
            &horizons,
            AgeRangeId::Under1H,
        );
        let mut lth = AggregateState::default();
        lth.add(
            Sats::from(200_u64),
            Sats::from(50_u64),
            Cents::from(3_000_u64),
            BoundedRatio::from(0.4),
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
}
