use std::iter;

use bitview_cohort::{AgeRange, AgeRangeId, UTXOAggregate};
use bitview_compute::AgeBand;
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_coinflow::HorizonId;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Day1, Sats, StoredF64, Version};
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, VecValue, WritableVec};

use super::Vecs;
use crate::{
    AgeCutoffs, Calibration, DayResult, DayUrpds, Dependencies, LossPercentileId, ModeId,
    ModeResult, ModeVecs, ModeWeights, PriceBandId, WRITE_INTERVAL_DAYS, WeightedModeId,
    WeightedModes, WeightedPair, WeightedUrpdNames,
};

const WEIGHTED_URPD_VERSION: Version = Version::TWO;

impl ModeVecs {
    fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.loss_threshold_stored
            .iter_mut()
            .map(|v| v as &mut dyn AnyStoredVec)
            .chain(
                self.prices_stored
                    .iter_mut()
                    .map(|v| v as &mut dyn AnyStoredVec),
            )
    }

    fn push(&mut self, result: &ModeResult) {
        for id in LossPercentileId::ALL {
            id.select_mut(&mut self.loss_threshold_stored)
                .push(*id.select(&result.loss_threshold));
        }
        for &id in PriceBandId::ALL {
            id.select_mut(&mut self.prices_stored)
                .push(*id.select(&result.prices));
        }
    }
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
            price,
            mappings,
            distribution,
            utxo_states,
            cointime,
            coinflow,
        } = dependencies;
        let exit = context.exit();
        let spot = &price.split.close.cents.day1;

        self.db.sync_bg_tasks()?;

        let cointime_wakefulness =
            AgeRange::from_fn(|id| &id.select(&cointime.age_range.activity.wakefulness).day1);
        let age_supplies = AgeRange::from_fn(|id| {
            &id.select(&distribution.cohorts.supply.total.cohorts.utxo.age)
                .sats
                .day1
        });
        let coinflow_mobility = AgeRange::from_fn(|id| {
            &id.select(&coinflow.age_range.spending_exposure.mobility)
                .day1
        });
        let coinflow_spending_rate =
            AgeRange::from_fn(|id| &id.select(&coinflow.age_range.spending_rate).day1);
        let raw_loss_share = &distribution
            .cohorts
            .relative
            .supply_profitability_shares
            .supply_in_loss_share
            .all
            .ppm
            .day1;
        let weighted_loss_shares =
            WeightedModes::from_fn(|mode| -> &dyn ReadableVec<Day1, Option<StoredF64>> {
                match mode {
                    WeightedModeId::Cointime => {
                        &cointime.supply.active_supply_in_loss_share.ratio.day1
                    }
                    WeightedModeId::Coinflow => &coinflow.all.supply_in_loss_share.day1,
                    WeightedModeId::Coinflow8Y => {
                        &coinflow.all.horizon._8y.supply_in_loss_share.day1
                    }
                    WeightedModeId::Coinflow4Y => {
                        &coinflow.all.horizon._4y.supply_in_loss_share.day1
                    }
                    WeightedModeId::Coinflow2Y => {
                        &coinflow.all.horizon._2y.supply_in_loss_share.day1
                    }
                    WeightedModeId::Coinflow1Y => {
                        &coinflow.all.horizon._1y.supply_in_loss_share.day1
                    }
                    WeightedModeId::Coinflow6M => {
                        &coinflow.all.horizon._6m.supply_in_loss_share.day1
                    }
                    WeightedModeId::Coinflow3M => {
                        &coinflow.all.horizon._3m.supply_in_loss_share.day1
                    }
                    WeightedModeId::Coinflow1M => {
                        &coinflow.all.horizon._1m.supply_in_loss_share.day1
                    }
                }
            });
        let weighted_urpd_names = DayUrpds::names();

        let weighted_urpd_source_version: Version = iter::once(WEIGHTED_URPD_VERSION)
            .chain(iter::once(mappings.day1.date.version()))
            .chain(iter::once(distribution.supply_state.version()))
            .chain(age_supplies.iter().map(|vec| vec.version()))
            .chain(cointime_wakefulness.iter().map(|vec| vec.version()))
            .chain(coinflow_mobility.iter().map(|vec| vec.version()))
            .sum();
        let source_version = Version::combine_all(
            iter::once(weighted_urpd_source_version)
                .chain(coinflow_spending_rate.iter().map(|vec| vec.version()))
                .chain(iter::once(raw_loss_share.version()))
                .chain(weighted_loss_shares.iter().map(|vec| vec.version())),
        );

        for vec in self.model_stored_vecs_mut() {
            vec.any_validate_computed_version_or_reset(source_version)?;
        }
        for vec in self.cost_basis.stored_vecs_mut() {
            vec.any_validate_computed_version_or_reset(
                weighted_urpd_source_version + spot.version(),
            )?;
        }
        for vec in self.capitalized_price.stored_vecs_mut() {
            vec.any_validate_computed_version_or_reset(weighted_urpd_source_version)?;
        }

        let source_end = iter::once(mappings.day1.date.len())
            .chain(iter::once(spot.len()))
            .chain(iter::once(raw_loss_share.len()))
            .chain(weighted_loss_shares.iter().map(|vec| vec.len()))
            .chain(age_supplies.iter().map(|vec| vec.len()))
            .chain(cointime_wakefulness.iter().map(|vec| vec.len()))
            .chain(coinflow_mobility.iter().map(|vec| vec.len()))
            .chain(coinflow_spending_rate.iter().map(|vec| vec.len()))
            .min()
            .unwrap_or_default();
        let recompute_from = mappings
            .height
            .recompute_day(indexer.safe_lengths().height)
            .map(usize::from)
            .unwrap_or_default();
        let weighted_urpd_is_current =
            DayUrpds::stored_version(&self.states_path)? == Some(weighted_urpd_source_version);
        if !weighted_urpd_is_current {
            DayUrpds::reset(&self.states_path, &weighted_urpd_names)?;
        }
        let weighted_urpd_start = if weighted_urpd_is_current {
            recompute_from
        } else {
            0
        };
        let model_start = self
            .model_minimum_len()
            .min(recompute_from)
            .min(weighted_urpd_start)
            .min(source_end);
        let cost_basis_start = self
            .cost_basis
            .minimum_len()
            .min(recompute_from)
            .min(weighted_urpd_start)
            .min(source_end);
        let capitalized_start = self
            .capitalized_price
            .minimum_len()
            .min(recompute_from)
            .min(weighted_urpd_start)
            .min(source_end);

        for vec in self.model_stored_vecs_mut() {
            vec.any_truncate_if_needed_at(model_start)?;
        }
        for vec in self.cost_basis.stored_vecs_mut() {
            vec.any_truncate_if_needed_at(cost_basis_start)?;
        }
        for vec in self.capitalized_price.stored_vecs_mut() {
            vec.any_truncate_if_needed_at(capitalized_start)?;
        }

        let age_bounds_version = Version::combine_all(
            iter::once(mappings.day1.date.version())
                .chain(iter::once(distribution.supply_state.version()))
                .chain(age_supplies.iter().map(|vec| vec.version())),
        );
        let age_bounds_start =
            self.cost_basis
                .age_bounds
                .prepare(age_bounds_version, recompute_from, source_end)?;
        if age_bounds_start < model_start {
            self.cost_basis.age_bounds.backfill(
                &mappings.day1.date,
                &distribution.states_path,
                age_bounds_start,
                model_start,
                exit,
            )?;
        }

        if capitalized_start < model_start {
            debug_assert!(weighted_urpd_is_current);
            self.backfill_capitalized_prices(
                mappings,
                &weighted_urpd_names,
                capitalized_start,
                model_start,
                exit,
            )?;
        }

        if cost_basis_start < model_start {
            debug_assert!(weighted_urpd_is_current);
            // Reuse version-validated weighted URPDs without rebuilding the Bedrock models.
            self.backfill_cost_basis(
                mappings,
                spot,
                &weighted_urpd_names,
                cost_basis_start,
                model_start,
                exit,
            )?;
        }

        let age_density_start = self.cost_basis.age_density.prepare(
            weighted_urpd_source_version + spot.version(),
            recompute_from.min(weighted_urpd_start),
            source_end,
        )?;

        let mut calibration =
            Calibration::from_sources(raw_loss_share, &weighted_loss_shares, model_start);
        let bounds = AgeBand::all();

        // Earlier dates only backfill age densities; the other independent groups
        // have already been brought up to model_start above.
        for day_index in model_start.min(age_density_start)..source_end {
            let day = Day1::from(day_index);
            let loss_shares = Calibration::loss_shares(raw_loss_share, &weighted_loss_shares, day);
            let thresholds = calibration.thresholds(&loss_shares);
            let mut result = DayResult::from_thresholds(&thresholds);
            let mut cost_basis_data = WeightedPair::default();
            let mut capitalized_prices = Self::missing_capitalized_prices();
            let mut age_price_bounds = AgeCutoffs::default();

            let needs_model = day_index >= model_start;
            let needs_evaluation = needs_model && thresholds.iter().any(Option::is_some);
            let needs_age_density = day_index >= age_density_start;
            let mut age_density = AgeCutoffs::default();
            let needs_rebuild = !weighted_urpd_is_current || day_index >= recompute_from;
            let needs_cost_basis = needs_model && day_index >= cost_basis_start;
            let needs_capitalized = needs_model && day_index >= capitalized_start;
            let needs_age_bounds = needs_model && day_index >= age_bounds_start;
            if let Some(date) = mappings.day1.date.collect_one(day)
                && (needs_rebuild
                    || needs_evaluation
                    || needs_cost_basis
                    || needs_capitalized
                    || needs_age_bounds
                    || needs_age_density)
            {
                let weights = Self::mode_weights(
                    day,
                    &age_supplies,
                    &cointime_wakefulness,
                    &coinflow_mobility,
                    &coinflow_spending_rate,
                    &bounds,
                );
                let urpds = if day_index + 1 == source_end {
                    Some(DayUrpds::current(utxo_states, &weights))
                } else {
                    DayUrpds::read_if_exists(&distribution.states_path, date, &weights)?
                };

                if let Some(urpds) = urpds {
                    age_price_bounds = urpds.age_price_bounds;
                    if needs_rebuild {
                        urpds.write(&self.states_path, &weighted_urpd_names, date)?;
                    }
                    if needs_evaluation {
                        result.evaluate(&urpds);
                    }
                    if needs_cost_basis {
                        cost_basis_data =
                            urpds.cost_basis(spot.collect_one(day).flatten().unwrap_or(Cents::NAN));
                    }
                    if needs_age_density {
                        age_density = urpds
                            .age_densities(spot.collect_one(day).flatten().unwrap_or(Cents::NAN));
                    }
                    if needs_capitalized {
                        capitalized_prices = urpds.capitalized_prices();
                    }
                }
            }
            if needs_model {
                calibration.observe(loss_shares);
            }
            if needs_age_density {
                self.cost_basis.age_density.push(&age_density);
            }

            if needs_age_bounds {
                self.cost_basis.age_bounds.push(&age_price_bounds);
            }
            if needs_cost_basis {
                self.cost_basis.push(&cost_basis_data);
            }
            if needs_capitalized {
                self.capitalized_price.push(&capitalized_prices);
            }

            if needs_model {
                for mode in ModeId::ALL {
                    self.modes
                        .select_mut(mode)
                        .push(result.by_mode.select(mode));
                }
            }

            if (day_index + 1).is_multiple_of(WRITE_INTERVAL_DAYS) || day_index + 1 == source_end {
                let _lock = exit.lock();
                if needs_model {
                    for vec in self.model_stored_vecs_mut() {
                        vec.write()?;
                    }
                }
                if needs_age_density {
                    for vec in self.cost_basis.age_density.stored_vecs_mut() {
                        vec.write()?;
                    }
                }
                if needs_age_bounds {
                    for vec in self.cost_basis.age_bounds.stored_vecs_mut() {
                        vec.write()?;
                    }
                }
                if needs_cost_basis {
                    for vec in self.cost_basis.stored_vecs_mut() {
                        vec.write()?;
                    }
                }
                if needs_capitalized {
                    for vec in self.capitalized_price.stored_vecs_mut() {
                        vec.write()?;
                    }
                }
            }
        }

        if !weighted_urpd_is_current {
            let _lock = exit.lock();
            DayUrpds::write_version(&self.states_path, weighted_urpd_source_version)?;
        }

        context.compact_database(&self.db);

        Ok(())
    }
}

impl Vecs {
    fn backfill_capitalized_prices(
        &mut self,
        mappings: &MappingsVecs,
        names: &WeightedUrpdNames,
        start: usize,
        end: usize,
        exit: &Exit,
    ) -> Result<()> {
        for day_index in start..end {
            let prices = if let Some(date) = mappings.day1.date.collect_one(Day1::from(day_index)) {
                DayUrpds::read_capitalized_prices(&self.states_path, names, date)?
            } else {
                Self::missing_capitalized_prices()
            };
            self.capitalized_price.push(&prices);
            if (day_index + 1).is_multiple_of(WRITE_INTERVAL_DAYS) || day_index + 1 == end {
                let _lock = exit.lock();
                for vec in self.capitalized_price.stored_vecs_mut() {
                    vec.write()?;
                }
            }
        }
        Ok(())
    }

    fn missing_capitalized_prices() -> UTXOAggregate<WeightedPair<Cents>> {
        UTXOAggregate {
            all: WeightedPair::from_fn(|_| Cents::NAN),
            sth: WeightedPair::from_fn(|_| Cents::NAN),
            lth: WeightedPair::from_fn(|_| Cents::NAN),
        }
    }

    fn backfill_cost_basis(
        &mut self,
        mappings: &MappingsVecs,
        spot: &impl ReadableVec<Day1, Option<Cents>>,
        names: &WeightedUrpdNames,
        start: usize,
        end: usize,
        exit: &Exit,
    ) -> Result<()> {
        for day_index in start..end {
            let day = Day1::from(day_index);
            let prices = if let Some(date) = mappings.day1.date.collect_one(day) {
                DayUrpds::read_cost_basis_if_exists(
                    &self.states_path,
                    names,
                    date,
                    spot.collect_one(day).flatten().unwrap_or(Cents::NAN),
                )?
                .unwrap_or_default()
            } else {
                WeightedPair::default()
            };
            self.cost_basis.push(&prices);

            if (day_index + 1).is_multiple_of(WRITE_INTERVAL_DAYS) || day_index + 1 == end {
                let _lock = exit.lock();
                for vec in self.cost_basis.stored_vecs_mut() {
                    vec.write()?;
                }
            }
        }
        Ok(())
    }

    fn model_stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.modes.iter_mut().flat_map(ModeVecs::stored_vecs_mut)
    }

    fn model_minimum_len(&mut self) -> usize {
        self.model_stored_vecs_mut()
            .map(|vec| vec.len())
            .min()
            .unwrap_or_default()
    }

    fn mode_weights(
        day: Day1,
        age_supplies: &AgeRange<&impl ReadableVec<Day1, Option<Sats>>>,
        cointime_wakefulness: &AgeRange<&impl ReadableVec<Day1, Option<StoredF64>>>,
        coinflow_mobility: &AgeRange<&impl ReadableVec<Day1, Option<StoredF64>>>,
        coinflow_spending_rate: &AgeRange<&impl ReadableVec<Day1, Option<StoredF64>>>,
        bounds: &AgeRange<AgeBand>,
    ) -> ModeWeights {
        debug_assert_eq!(
            WeightedModeId::COINFLOW_HORIZONS.len(),
            HorizonId::ALL.len()
        );
        let mut weights = ModeWeights::from_fn(|_| None);
        weights.raw = Some(AgeRange::from_fn(|_| 1.0));

        let Some(supplies) = Self::collect_age_supplies(age_supplies, day) else {
            return weights;
        };
        weights.cointime = Self::collect_age_values(cointime_wakefulness, &supplies, day)
            .map(|values| AgeRange::from_fn(|id| (*id.select(&values)).clamp(0.0, 1.0)));
        weights.coinflow = Self::collect_age_values(coinflow_mobility, &supplies, day)
            .map(|values| AgeRange::from_fn(|id| (*id.select(&values)).clamp(0.0, 1.0)));

        if let Some(hazards) = Self::collect_age_values(coinflow_spending_rate, &supplies, day) {
            let hazards = AgeRange::from_fn(|id| (*id.select(&hazards)).max(0.0));
            for (id, horizon) in WeightedModeId::COINFLOW_HORIZONS
                .into_iter()
                .zip(HorizonId::ALL.into_iter().map(HorizonId::days))
            {
                *weights.select_mut(id.mode()) = Some(AgeRange::from_fn(|age| {
                    AgeBand::horizon_mobility(&hazards, age, horizon, bounds)
                }));
            }
        }
        weights
    }

    fn collect_age_supplies(
        sources: &AgeRange<&impl ReadableVec<Day1, Option<Sats>>>,
        day: Day1,
    ) -> Option<AgeRange<Sats>> {
        let mut supplies = AgeRange::default();
        for &id in AgeRangeId::ALL {
            *id.select_mut(&mut supplies) = id.select(sources).collect_one(day).flatten()?;
        }
        Some(supplies)
    }

    fn collect_age_values<T>(
        sources: &AgeRange<&impl ReadableVec<Day1, Option<T>>>,
        supplies: &AgeRange<Sats>,
        day: Day1,
    ) -> Option<AgeRange<f64>>
    where
        T: VecValue,
        f64: From<T>,
    {
        let mut values = AgeRange::default();
        for &id in AgeRangeId::ALL {
            *id.select_mut(&mut values) = Self::resolve_age_value(
                id.select(sources).collect_one(day).flatten(),
                *id.select(supplies),
            )?;
        }
        Some(values)
    }
}
