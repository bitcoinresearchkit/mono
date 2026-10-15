use std::{
    mem,
    time::{Duration, Instant},
};

use bitview_cohort::AgeRange;
use bitview_compute::{Inputs, WRITE_INTERVAL, collect_cohort_weights, write_locked};
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_traversable::Traversable;
use bitview_urpd::COMPUTE_VERSION as URPD_COMPUTE_VERSION;
use brk_error::Result;
use log::info;
use vecdb::{AnyVec, ReadableVec};

use crate::{BlockResult, Calibration, CumulativeBucket, Dependencies, ModeId, Vecs};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies {
            urpd,
            indexer,
            age,
            holders: _,
            cointime,
            coinflow,
        } = dependencies;

        let cointime_wakefulness = cointime.age_ranges.urpd_weight_sources();
        let age_supplies = age.age_supplies();
        let coinflow_mobility = coinflow.age_ranges.urpd_weight_sources();
        let raw_loss_share = dependencies.raw_loss_share();
        let cointime_loss_share = dependencies.cointime_loss_share();
        let coinflow_loss_share = dependencies.coinflow_loss_share();
        let mut inputs = Inputs::new(
            urpd.timestamps.version() + URPD_COMPUTE_VERSION,
            [
                urpd.prices as &dyn AnyVec,
                age.all_supply(),
                raw_loss_share,
                cointime_loss_share,
                coinflow_loss_share,
            ]
            .into_iter()
            .chain(age_supplies.iter().map(|vec| *vec as &dyn AnyVec))
            .chain(cointime_wakefulness.iter().map(|vec| *vec as &dyn AnyVec))
            .chain(coinflow_mobility.iter().map(|vec| *vec as &dyn AnyVec)),
        );
        inputs.len = inputs.len.min(urpd.timestamps.len());
        let source_version = inputs.version;
        let range = inputs.prepare(
            self.modes.iter_any_stored_mut().collect::<Vec<_>>(),
            usize::from(indexer.compute_start().height),
            context.exit(),
        )?;
        let (start, end) = (range.start, range.end);
        if end - start >= WRITE_INTERVAL {
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
            let mut raw = raw_loss_share.bounded_cursor_at(start, end);
            let mut cointime = cointime_loss_share.bounded_cursor_at(start, end);
            let mut coinflow = coinflow_loss_share.bounded_cursor_at(start, end);
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
            |blocks| {
                let (from, to) = (blocks.start, blocks.end);
                Worker {
                    supplies: AgeRange::from_fn(|age| {
                        age.select(&age_supplies).bounded_cursor_at(from, to)
                    }),
                    cointime: AgeRange::from_fn(|age| {
                        age.select(&cointime_wakefulness)
                            .bounded_cursor_at(from, to)
                    }),
                    coinflow: AgeRange::from_fn(|age| {
                        age.select(&coinflow_mobility).bounded_cursor_at(from, to)
                    }),
                    scratch: Vec::new(),
                }
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
                    result.evaluate(
                        source,
                        &[ct_weights.as_ref(), cf_weights.as_ref()],
                        &mut worker.scratch,
                    );
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
                if end_block.is_multiple_of(WRITE_INTERVAL) || end_block == end {
                    write_locked(self.modes.iter_any_stored_mut(), context.exit())?;
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
