use std::{
    mem,
    time::{Duration, Instant},
};

use bitview_cohort::AgeRange;
use bitview_compute::{Inputs, WRITE_INTERVAL, collect_cohort_weights, write_locked};
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_traversable::Traversable;
use bitview_urpd::{COMPUTE_VERSION as URPD_COMPUTE_VERSION, ReplayInputs};
use brk_error::Result;
use brk_types::Version;
use log::info;
use vecdb::{AnyVec, ReadableVec};

use crate::{BlockResult, Calibration, CumulativeBucket, Dependencies, ModeVecs, Vecs};

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
            holders,
            cointime,
            coinflow,
        } = dependencies;

        let cointime_wakefulness = cointime.age_ranges.urpd_weight_sources();
        let age_supplies = age.age_supplies();
        let coinflow_mobility = coinflow.age_ranges.urpd_weight_sources();
        let raw_loss_share = &holders.cohorts.all.supply.in_loss.share.fixed.height;
        let cointime_loss_share = cointime.all_awake_supply_in_loss_share();
        let coinflow_loss_share = coinflow.all_mobile_supply_in_loss_share();
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
            self.modes
                .iter_mut()
                .flat_map(ModeVecs::replay_stored_mut)
                .collect::<Vec<_>>(),
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
                    let supplies = AgeRange::from_fn(|age| {
                        age.select_mut(&mut worker.supplies)
                            .get(index)
                            .expect("validated age supply range")
                    });
                    let ct_weights =
                        collect_cohort_weights(height, &mut worker.cointime, &supplies);
                    let cf_weights =
                        collect_cohort_weights(height, &mut worker.coinflow, &supplies);
                    result.evaluate(
                        source,
                        &[ct_weights.as_ref(), cf_weights.as_ref()],
                        &mut worker.scratch,
                    );
                }
                Ok(result)
            },
            |height, result| {
                for (vecs, result) in self.modes.iter_mut().zip(result.iter()) {
                    vecs.push(result);
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
                    write_locked(
                        self.modes.iter_mut().flat_map(ModeVecs::replay_stored_mut),
                        context.exit(),
                    )?;
                }
                Ok(())
            },
        )?;
        self.calibration = Some(calibration);
        self.replay = replay;
        self.compute_floor_supply(start, urpd, context)
    }
}

impl Vecs {
    /// The floor bands' holdings, from the floors the replay stored: their own group, so each
    /// resumes from where it was written.
    fn compute_floor_supply(
        &mut self,
        max_from: usize,
        urpd: ReplayInputs<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let floors = Inputs::new(
            Version::ZERO,
            self.modes
                .iter()
                .flat_map(|vecs| vecs.prices_stored.floor.iter())
                .map(|vec| vec as &dyn AnyVec)
                .chain([urpd.prices as &dyn AnyVec]),
        );
        let range = floors.prepare(
            self.modes
                .iter_mut()
                .flat_map(|vecs| vecs.floor.iter_any_stored_mut())
                .collect::<Vec<_>>(),
            max_from,
            context.exit(),
        )?;
        let mut floor_supply = mem::take(&mut self.floor_supply);
        floor_supply.resume(&range, &self.modes, urpd)?;
        floor_supply.compute(range, urpd.history, &mut self.modes, context.exit())?;
        self.floor_supply = floor_supply;
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
