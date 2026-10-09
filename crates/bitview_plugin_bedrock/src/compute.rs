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
use vecdb::{AnyStoredVec, AnyVec, Database, ReadableVec};

use crate::{
    BlockResult, Calibration, CumulativeBucket, Dependencies, ModeId, ModeVecs, Vecs,
    WRITE_INTERVAL_BLOCKS, WeightedModeId, WeightedModes,
};

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies {
            urpd,
            indexer,
            mappings,
            age,
            holders: _,
            cointime,
            coinflow,
        } = dependencies;

        let cointime_wakefulness = cointime.age_range.urpd_weight_sources();
        let age_supplies = age.cohorts.supply.total.age_supplies();
        let coinflow_mobility = coinflow.age_range.urpd_weight_sources();
        let raw_loss_share = dependencies.raw_loss_share();
        let cointime_loss_share = dependencies.cointime_loss_share();
        let coinflow_loss_share = dependencies.coinflow_loss_share();
        let source_version = Version::combine_all(
            iter::once(mappings.timestamp.monotonic.version() + URPD_COMPUTE_VERSION)
                .chain(iter::once(urpd.prices.version()))
                .chain(iter::once(age.cohorts.all_supply().version()))
                .chain(iter::once(raw_loss_share.version()))
                .chain(iter::once(cointime_loss_share.version()))
                .chain(iter::once(coinflow_loss_share.version()))
                .chain(age_supplies.iter().map(|vec| vec.version()))
                .chain(cointime_wakefulness.iter().map(|vec| vec.version()))
                .chain(coinflow_mobility.iter().map(|vec| vec.version())),
        );
        let end = iter::once(mappings.timestamp.monotonic.len())
            .chain(iter::once(raw_loss_share.len()))
            .chain(iter::once(cointime_loss_share.len()))
            .chain(iter::once(coinflow_loss_share.len()))
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
        let mut calibration = self
            .calibration
            .take()
            .filter(|c| c.end == start && c.version == source_version)
            .unwrap_or_else(|| {
                Calibration::from_sources(
                    raw_loss_share,
                    cointime_loss_share,
                    coinflow_loss_share,
                    start,
                    source_version,
                )
            });
        // Thresholds follow the stored loss shares alone, so one sequential pass fixes them
        // and the URPD evaluation can replay in parallel.
        let thresholds = {
            let mut raw = raw_loss_share.cursor();
            let mut cointime = cointime_loss_share.cursor();
            let mut coinflow = coinflow_loss_share.cursor();
            (start..end)
                .map(|index| {
                    let shares = Calibration::loss_shares(
                        raw.get(index).map(f64::from),
                        cointime.get(index).map(f64::from),
                        coinflow.get(index).map(f64::from),
                    );
                    let thresholds = calibration.thresholds(&shares);
                    calibration.observe(shares);
                    thresholds
                })
                .collect::<Vec<_>>()
        };
        let mut last_progress = Instant::now();
        let mut replay = mem::take(&mut self.replay);
        replay.map(
            start..end,
            urpd,
            || Worker {
                supplies: AgeRange::from_fn(|age| age.select(&age_supplies).cursor()),
                cointime: AgeRange::from_fn(|age| age.select(&cointime_wakefulness).cursor()),
                coinflow: AgeRange::from_fn(|age| age.select(&coinflow_mobility).cursor()),
                scratch: Vec::new(),
            },
            |worker, height, _, source| {
                let index = usize::from(height);
                let thresholds = &thresholds[index - start];
                let mut result = BlockResult::from_thresholds(thresholds);
                if thresholds.iter().any(Option::is_some) {
                    let supplies = AgeRange::try_from_fn(|age| {
                        age.select_mut(&mut worker.supplies).get(index).ok_or(())
                    })
                    .ok();
                    let ct_weights = supplies
                        .as_ref()
                        .and_then(|s| collect_cohort_weights(height, &mut worker.cointime, s));
                    let cf_weights = supplies
                        .as_ref()
                        .and_then(|s| collect_cohort_weights(height, &mut worker.coinflow, s));
                    let weights = WeightedModes::from_fn(|mode| match mode {
                        WeightedModeId::Cointime => ct_weights.as_ref(),
                        WeightedModeId::Coinflow => cf_weights.as_ref(),
                    });
                    result.evaluate(source, &weights, &mut worker.scratch);
                }
                Ok(result)
            },
            |height, result| {
                for mode in ModeId::ALL {
                    self.modes
                        .select_mut(mode)
                        .push(result.by_mode.select(mode));
                }
                let end_block = usize::from(height) + 1;
                if end_block.is_multiple_of(1_000)
                    && last_progress.elapsed() >= Duration::from_secs(10)
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
            },
        )?;
        self.calibration = Some(calibration);
        self.replay = replay;
        Ok(())
    }
}

/// One segment's cursors and cumulative-bucket scratch.
struct Worker<S, T, F> {
    supplies: AgeRange<S>,
    cointime: AgeRange<T>,
    coinflow: AgeRange<F>,
    scratch: Vec<CumulativeBucket>,
}

impl Vecs {
    fn model_stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.modes.iter_mut().flat_map(ModeVecs::stored_vecs_mut)
    }
}
