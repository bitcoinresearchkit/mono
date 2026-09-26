use std::iter;

use bitview_cohort::AgeRange;
use bitview_compute::prepare_computed;
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_coinflow::Vecs as CoinflowVecs;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_urpd::COMPUTE_VERSION as URPD_COMPUTE_VERSION;
use brk_error::Result;
use brk_types::{Day1, Sats, StoredF64, Version};
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, WritableVec};

use super::Vecs;
use crate::{
    Calibration, DayResult, DayUrpds, Dependencies, LossPercentileId, ModeId, ModeResult, ModeVecs,
    ModeWeights, PriceBandId, WRITE_INTERVAL_DAYS, WeightedModeId, WeightedModes,
};

impl ModeVecs {
    fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.supply_in_loss_threshold_stored
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
            id.select_mut(&mut self.supply_in_loss_threshold_stored)
                .push(*id.select(&result.supply_in_loss_threshold));
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
            mappings,
            distribution,
            age_urpds,
            cointime,
            coinflow,
        } = dependencies;
        let exit = context.exit();

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
                    _ => {
                        &mode
                            .coinflow_horizon()
                            .unwrap()
                            .select(&coinflow.all.horizon)
                            .supply_in_loss_share
                            .day1
                    }
                }
            });
        let source_version = Version::combine_all(
            iter::once(mappings.day1.date.version() + URPD_COMPUTE_VERSION)
                .chain(iter::once(distribution.supply_state.version()))
                .chain(iter::once(raw_loss_share.version()))
                .chain(weighted_loss_shares.iter().map(|vec| vec.version()))
                .chain(age_supplies.iter().map(|vec| vec.version()))
                .chain(cointime_wakefulness.iter().map(|vec| vec.version()))
                .chain(coinflow_mobility.iter().map(|vec| vec.version()))
                .chain(coinflow_spending_rate.iter().map(|vec| vec.version())),
        );
        let end = iter::once(mappings.day1.date.len())
            .chain(iter::once(raw_loss_share.len()))
            .chain(weighted_loss_shares.iter().map(|vec| vec.len()))
            .chain(age_supplies.iter().map(|vec| vec.len()))
            .chain(cointime_wakefulness.iter().map(|vec| vec.len()))
            .chain(coinflow_mobility.iter().map(|vec| vec.len()))
            .chain(coinflow_spending_rate.iter().map(|vec| vec.len()))
            .min()
            .unwrap_or_default();
        let from = mappings
            .height
            .recompute_day(indexer.safe_lengths().height)
            .map(usize::from)
            .unwrap_or_default()
            .min(end);
        let start = prepare_computed(
            self.model_stored_vecs_mut().collect::<Vec<_>>(),
            source_version,
            from,
        )?;
        let mut calibration =
            Calibration::from_sources(raw_loss_share, &weighted_loss_shares, start);
        for index in start..end {
            let day = Day1::from(index);
            let shares = Calibration::loss_shares(raw_loss_share, &weighted_loss_shares, day);
            let thresholds = calibration.thresholds(&shares);
            let mut result = DayResult::from_thresholds(&thresholds);
            if thresholds.iter().any(Option::is_some)
                && let Some(date) = mappings.day1.date.collect_one(day)
            {
                let weights = Self::mode_weights(day, &age_supplies, cointime, coinflow);
                if let Some(urpds) = age_urpds.with_entries(
                    &distribution.states_path,
                    date,
                    index + 1 == mappings.day1.date.len(),
                    |entries| DayUrpds::from_age_entries(entries, &weights),
                )? {
                    result.evaluate(&urpds);
                }
            }
            calibration.observe(shares);
            for mode in ModeId::ALL {
                self.modes
                    .select_mut(mode)
                    .push(result.by_mode.select(mode));
            }
            if (index + 1).is_multiple_of(WRITE_INTERVAL_DAYS) || index + 1 == end {
                let _lock = exit.lock();
                for vec in self.model_stored_vecs_mut() {
                    vec.write()?;
                }
            }
        }

        context.compact_database(&self.db);

        Ok(())
    }
}

impl Vecs {
    fn model_stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.modes.iter_mut().flat_map(ModeVecs::stored_vecs_mut)
    }

    fn mode_weights(
        day: Day1,
        age_supplies: &AgeRange<&impl ReadableVec<Day1, Option<Sats>>>,
        cointime: &CointimeVecs,
        coinflow: &CoinflowVecs,
    ) -> ModeWeights {
        let mut weights = ModeWeights::from_fn(|_| None);
        weights.raw = Some(AgeRange::from_fn(|_| 1.0));
        let Ok(supplies) = AgeRange::try_from_fn(|age| {
            age.select(age_supplies)
                .collect_one(day)
                .flatten()
                .ok_or(())
        }) else {
            return weights;
        };
        weights.cointime = AgeRange::try_from_fn(|age| {
            cointime
                .urpd_weight(age, day, *age.select(&supplies))
                .ok_or(())
        })
        .ok();
        weights.coinflow = AgeRange::try_from_fn(|age| {
            coinflow
                .urpd_weight(age, day, *age.select(&supplies))
                .ok_or(())
        })
        .ok();
        if let Some(horizons) = coinflow.horizon_weights(day, &supplies) {
            for id in WeightedModeId::COINFLOW_HORIZONS {
                *weights.select_mut(id.mode()) =
                    Some(id.coinflow_horizon().unwrap().select(&horizons).clone());
            }
        }
        weights
    }
}
