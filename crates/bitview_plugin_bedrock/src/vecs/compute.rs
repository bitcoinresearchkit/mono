use std::{
    iter, mem,
    time::{Duration, Instant},
};

use bitview_cohort::AgeRange;
use bitview_compute::{collect_cohort_weights, prepare_computed};
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_urpd::COMPUTE_VERSION as URPD_COMPUTE_VERSION;
use brk_error::Result;
use brk_types::Version;
use log::info;
use vecdb::{AnyStoredVec, AnyVec, ReadableVec};

use super::Vecs;
use crate::{
    BlockResult, Calibration, Dependencies, ModeId, ModeVecs, WRITE_INTERVAL_BLOCKS,
    WeightedModeId, WeightedModes,
};

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies {
            urpd,
            indexer,
            mappings,
            distribution_age,
            distribution_aggregated: _,
            cointime,
            coinflow,
        } = dependencies;
        self.db.sync_bg_tasks()?;

        let cointime_wakefulness =
            AgeRange::from_fn(|id| &id.select(&cointime.age_range.activity.wakefulness).height);
        let age_supplies = distribution_age.cohorts.supply.total.age_supplies();
        let coinflow_mobility = AgeRange::from_fn(|id| {
            &id.select(&coinflow.age_range.spending_exposure.mobility)
                .height
        });
        let raw_loss_share = dependencies.raw_loss_share();
        let weighted_loss_shares = dependencies.weighted_loss_shares();
        let source_version = Version::combine_all(
            iter::once(mappings.timestamp.monotonic.version() + URPD_COMPUTE_VERSION)
                .chain(iter::once(urpd.prices.version()))
                .chain(iter::once(distribution_age.cohorts.all_supply().version()))
                .chain(iter::once(raw_loss_share.version()))
                .chain(weighted_loss_shares.iter().map(|vec| vec.version()))
                .chain(age_supplies.iter().map(|vec| vec.version()))
                .chain(cointime_wakefulness.iter().map(|vec| vec.version()))
                .chain(coinflow_mobility.iter().map(|vec| vec.version())),
        );
        let end = iter::once(mappings.timestamp.monotonic.len())
            .chain(iter::once(raw_loss_share.len()))
            .chain(weighted_loss_shares.iter().map(|vec| vec.len()))
            .chain(age_supplies.iter().map(|vec| vec.len()))
            .chain(cointime_wakefulness.iter().map(|vec| vec.len()))
            .chain(coinflow_mobility.iter().map(|vec| vec.len()))
            .min()
            .unwrap_or_default();
        let from = usize::from(indexer.safe_lengths().height).min(end);
        let start = prepare_computed(
            self.model_stored_vecs_mut().collect::<Vec<_>>(),
            source_version,
            from,
            context.exit(),
        )?;
        if end.saturating_sub(start) >= WRITE_INTERVAL_BLOCKS {
            info!(
                "Computing Bedrock for {} blocks ({start}..={})...",
                end - start,
                end - 1
            );
        }
        let mut last_progress = Instant::now();
        let mut replay = mem::take(&mut self.replay);
        let mut scratch = mem::take(&mut self.scratch);
        let mut calibration = self
            .calibration
            .take()
            .filter(|c| c.end == start && c.version == source_version)
            .unwrap_or_else(|| {
                Calibration::from_sources(
                    raw_loss_share,
                    &weighted_loss_shares,
                    start,
                    source_version,
                )
            });
        replay.for_each(start..end, urpd, |height, _, source| {
            let shares = Calibration::loss_shares(raw_loss_share, &weighted_loss_shares, height);
            let thresholds = calibration.thresholds(&shares);
            let mut result = BlockResult::from_thresholds(&thresholds);
            if thresholds.iter().any(Option::is_some) {
                let supplies = AgeRange::try_from_fn(|age| {
                    age.select(&age_supplies).collect_one(height).ok_or(())
                })
                .ok();
                let ct_weights = supplies
                    .as_ref()
                    .and_then(|s| collect_cohort_weights(height, &cointime_wakefulness, s));
                let cf_weights = supplies
                    .as_ref()
                    .and_then(|s| collect_cohort_weights(height, &coinflow_mobility, s));
                let weights = WeightedModes::from_fn(|mode| match mode {
                    WeightedModeId::Cointime => ct_weights.as_ref(),
                    WeightedModeId::Coinflow => cf_weights.as_ref(),
                });
                result.evaluate(source, &weights, &mut scratch);
            }
            calibration.observe(shares);
            for mode in ModeId::ALL {
                self.modes
                    .select_mut(mode)
                    .push(result.by_mode.select(mode));
            }
            let end_block = usize::from(height) + 1;
            if end_block.is_multiple_of(1_000) && last_progress.elapsed() >= Duration::from_secs(10)
            {
                info!(
                    "Computing Bedrock: block {height}/{}, {}/{} blocks",
                    end - 1,
                    end_block - start,
                    end - start
                );
                last_progress = Instant::now();
            }
            if end_block.is_multiple_of(WRITE_INTERVAL_BLOCKS) || end_block == end {
                let _lock = context.exit().lock();
                for vec in self.model_stored_vecs_mut() {
                    vec.write()?;
                }
            }
            Ok(())
        })?;
        self.calibration = Some(calibration);
        self.replay = replay;
        self.scratch = scratch;
        context.compact_database(&self.db);
        Ok(())
    }
}

impl Vecs {
    fn model_stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.modes.iter_mut().flat_map(ModeVecs::stored_vecs_mut)
    }
}
