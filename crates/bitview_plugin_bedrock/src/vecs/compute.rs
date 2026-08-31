use brk_error::Result;

use bitview_cohort::{AgeRange, AgeRangeId};
use bitview_compute::{AgeBand, db_utils::validate_any_computed_version_or_reset};
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_coinflow::HorizonId;
use bitview_plugin_distribution::UTXOStates;
use bitview_plugin_indexer::Indexer;
use brk_exit::Exit;
use brk_types::{
    Cents, CostBasisPercentilePrices, Day1, PERCENTILES_LEN, Sats, StoredF64, Version,
};
use vecdb::{AnyStoredVec, AnyVec, ColumnId, ReadableVec, VecValue};

use super::Vecs;
use crate::{
    Calibration, DayResult, DayUrpds, Dependencies, LossPercentileId, ModeId, ModeResult, ModeVecs,
    ModeWeights, PriceBandId, WeightedModeId, WeightedModes, WeightedPair,
};

const WRITE_INTERVAL_DAYS: usize = 100;
const WEIGHTED_URPD_VERSION: Version = Version::TWO;

impl ModeVecs {
    fn stored_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 2] {
        [self.loss_threshold.stored_mut(), self.prices.stored_mut()]
    }

    fn push(&mut self, result: &ModeResult) {
        self.loss_threshold.push(LossPercentileId::from_fn(|id| {
            *id.select(&result.loss_threshold)
        }));
        self.prices
            .push(PriceBandId::from_fn(|id| *id.select(&result.prices)));
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
        self.compute_inner(
            dependencies.indexer,
            dependencies.mappings,
            dependencies.distribution,
            dependencies.utxo_states,
            dependencies.cointime,
            dependencies.coinflow,
            context.exit(),
        )
    }
}

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    fn compute_inner(
        &mut self,
        indexer: &Indexer,
        mappings: &bitview_plugin_mappings::Vecs,
        distribution: &bitview_plugin_distribution::Vecs,
        utxo_states: &UTXOStates,
        cointime: &bitview_plugin_cointime::Vecs,
        coinflow: &bitview_plugin_coinflow::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        let cointime_wakefulness =
            AgeRange::from_fn(|id| &id.select(&cointime.age_range.activity.wakefulness).day1);
        let age_supplies = AgeRange::from_fn(|id| {
            &id.select(&distribution.cohorts.supply.total.cohorts.age.range)
                .sats
                .day1
                .0
        });
        let coinflow_mobility = AgeRange::from_fn(|id| {
            &id.select(&coinflow.age_range.spending_exposure.mobility)
                .day1
                .0
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
            .day1
            .0;
        let weighted_loss_shares =
            WeightedModes::from_fn(|mode| -> &dyn ReadableVec<Day1, Option<StoredF64>> {
                match mode {
                    WeightedModeId::Cointime => &cointime.supply.active_supply_in_loss_share.day1,
                    WeightedModeId::Coinflow => &coinflow.all.supply_in_loss_share.day1.0,
                    WeightedModeId::Coinflow8Y => {
                        &coinflow.all.horizon._8y.supply_in_loss_share.day1.0
                    }
                    WeightedModeId::Coinflow4Y => {
                        &coinflow.all.horizon._4y.supply_in_loss_share.day1.0
                    }
                    WeightedModeId::Coinflow2Y => {
                        &coinflow.all.horizon._2y.supply_in_loss_share.day1.0
                    }
                    WeightedModeId::Coinflow1Y => {
                        &coinflow.all.horizon._1y.supply_in_loss_share.day1.0
                    }
                    WeightedModeId::Coinflow6M => {
                        &coinflow.all.horizon._6m.supply_in_loss_share.day1.0
                    }
                    WeightedModeId::Coinflow3M => {
                        &coinflow.all.horizon._3m.supply_in_loss_share.day1.0
                    }
                    WeightedModeId::Coinflow1M => {
                        &coinflow.all.horizon._1m.supply_in_loss_share.day1.0
                    }
                }
            });
        let weighted_urpd_names = DayUrpds::names();

        let weighted_urpd_source_version: Version = std::iter::once(WEIGHTED_URPD_VERSION)
            .chain(std::iter::once(mappings.day1.date.version()))
            .chain(std::iter::once(distribution.supply_state.version()))
            .chain(age_supplies.iter().map(|vec| vec.version()))
            .chain(cointime_wakefulness.iter().map(|vec| vec.version()))
            .chain(coinflow_mobility.iter().map(|vec| vec.version()))
            .sum();
        let source_version = Version::combine_all(
            std::iter::once(weighted_urpd_source_version)
                .chain(coinflow_spending_rate.iter().map(|vec| vec.version()))
                .chain(std::iter::once(raw_loss_share.version()))
                .chain(weighted_loss_shares.iter().map(|vec| vec.version())),
        );

        for vec in self.model_stored_vecs_mut() {
            validate_any_computed_version_or_reset(vec, source_version)?;
        }
        for vec in self.cost_basis.stored_vecs_mut() {
            validate_any_computed_version_or_reset(vec, weighted_urpd_source_version)?;
        }

        let source_end = std::iter::once(mappings.day1.date.len())
            .chain(std::iter::once(raw_loss_share.len()))
            .chain(weighted_loss_shares.iter().map(|vec| vec.len()))
            .chain(age_supplies.iter().map(|vec| vec.len()))
            .chain(cointime_wakefulness.iter().map(|vec| vec.len()))
            .chain(coinflow_mobility.iter().map(|vec| vec.len()))
            .chain(coinflow_spending_rate.iter().map(|vec| vec.len()))
            .min()
            .unwrap_or_default();
        let recompute_from = Self::recompute_day(indexer, mappings)
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

        for vec in self.model_stored_vecs_mut() {
            vec.any_truncate_if_needed_at(model_start)?;
        }
        for vec in self.cost_basis.stored_vecs_mut() {
            vec.any_truncate_if_needed_at(cost_basis_start)?;
        }

        if cost_basis_start < model_start {
            debug_assert!(weighted_urpd_is_current);
            // Reuse version-validated weighted URPDs without rebuilding the Bedrock models.
            self.backfill_cost_basis(
                mappings,
                &weighted_urpd_names,
                cost_basis_start,
                model_start,
                exit,
            )?;
        }

        let mut calibration =
            Calibration::from_sources(raw_loss_share, &weighted_loss_shares, model_start);
        let bounds = AgeBand::all();

        for day_index in model_start..source_end {
            let day = Day1::from(day_index);
            let loss_shares = Calibration::loss_shares(raw_loss_share, &weighted_loss_shares, day);
            let thresholds = calibration.thresholds(&loss_shares);
            let mut result = DayResult::from_thresholds(&thresholds);
            let mut cost_basis_prices = Self::missing_cost_basis_prices();

            let needs_evaluation = thresholds.iter().any(Option::is_some);
            let needs_rebuild = !weighted_urpd_is_current || day_index >= recompute_from;
            let needs_cost_basis = day_index >= cost_basis_start;
            if let Some(date) = mappings.day1.date.collect_one(day)
                && (needs_rebuild || needs_evaluation || needs_cost_basis)
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
                    if needs_rebuild {
                        urpds.write(&self.states_path, &weighted_urpd_names, date)?;
                    }
                    if needs_evaluation {
                        result.evaluate(&urpds, &thresholds);
                    }
                    if needs_cost_basis {
                        cost_basis_prices = urpds.all_cost_basis_percentile_prices();
                    }
                }
            }
            calibration.observe(loss_shares);

            if needs_cost_basis {
                self.cost_basis.push(&cost_basis_prices);
            }

            for mode in ModeId::ALL {
                self.modes
                    .select_mut(mode)
                    .push(result.by_mode.select(mode));
            }

            if (day_index + 1).is_multiple_of(WRITE_INTERVAL_DAYS) || day_index + 1 == source_end {
                let _lock = exit.lock();
                for vec in self.model_stored_vecs_mut() {
                    vec.write()?;
                }
                if needs_cost_basis {
                    for vec in self.cost_basis.stored_vecs_mut() {
                        vec.write()?;
                    }
                }
            }
        }

        if !weighted_urpd_is_current {
            let _lock = exit.lock();
            DayUrpds::write_version(&self.states_path, weighted_urpd_source_version)?;
        }

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });

        Ok(())
    }

    fn backfill_cost_basis(
        &mut self,
        mappings: &bitview_plugin_mappings::Vecs,
        names: &crate::WeightedUrpdNames,
        start: usize,
        end: usize,
        exit: &Exit,
    ) -> Result<()> {
        for day_index in start..end {
            let day = Day1::from(day_index);
            let prices = if let Some(date) = mappings.day1.date.collect_one(day) {
                DayUrpds::read_all_cost_basis_percentile_prices_if_exists(
                    &self.states_path,
                    names,
                    date,
                )?
                .unwrap_or_else(Self::missing_cost_basis_prices)
            } else {
                Self::missing_cost_basis_prices()
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

    fn missing_cost_basis_prices() -> WeightedPair<CostBasisPercentilePrices> {
        WeightedPair::from_fn(|_| CostBasisPercentilePrices {
            per_coin: [Cents::NAN; PERCENTILES_LEN],
            per_dollar: [Cents::NAN; PERCENTILES_LEN],
        })
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

    fn recompute_day(indexer: &Indexer, mappings: &bitview_plugin_mappings::Vecs) -> Option<Day1> {
        let starting_height = indexer.safe_lengths().height;
        mappings
            .height
            .day1
            .collect_one(starting_height)
            .or_else(|| {
                starting_height
                    .decremented()
                    .and_then(|height| mappings.height.day1.collect_one(height))
            })
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
