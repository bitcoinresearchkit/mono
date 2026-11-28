use std::{
    mem,
    time::{Duration, Instant},
};

use bitview_cohort::AgeRange;
use bitview_compute::{
    Inputs, WRITE_INTERVAL, collect_cohort_weights, day_closes, prepare_computed, prepare_days,
    write_locked,
};
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_primitives::{Day1, Ratio};
use bitview_traversable::Traversable;
use bitview_urpd::COMPUTE_VERSION as URPD_COMPUTE_VERSION;
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use log::info;
use statedb::Reader;
use vecdb::{AnyVec, PcoVec, ReadableVec, VecIndex, WritableVec};

use crate::{BlockResult, Calibration, CumulativeBucket, Dependencies, ModeVecs, Modes, Vecs};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies {
            urpd,
            history,
            price,
            indexer,
            age,
            holders,
            cointime,
            coinflow,
        } = dependencies;

        let prices = &price.spot.cents.height;
        let cointime_wakefulness = cointime.age_ranges.urpd_weight_sources();
        let age_supplies = age.age_supplies();
        let coinflow_mobility = coinflow.age_ranges.urpd_weight_sources();
        let raw_loss_share = &holders.cohorts.all.supply.in_loss.share.fixed.height;
        let cointime_loss_share = cointime.all_awake_supply_in_loss_share();
        let coinflow_loss_share = coinflow.all_mobile_supply_in_loss_share();
        let mut inputs = Inputs::new(
            urpd.version() + URPD_COMPUTE_VERSION,
            [
                prices as &dyn AnyVec,
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
        inputs.len = inputs.len.min(urpd.len());
        let source_version = inputs.version;
        let rank_version = Version::ONE
            + source_version
            + self.day_of_height.version()
            + self.first_height.version();
        let ranked_days = prepare_computed(
            rank_vecs(&mut self.modes),
            rank_version,
            usize::MAX,
            context.exit(),
        )?;
        let range = inputs.prepare(
            self.modes
                .iter_mut()
                .flat_map(ModeVecs::walk_stored_mut)
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
        // The ranks resume on their own: with the walk, or before it where they are behind (new
        // or reset), observing those blocks without thresholds.
        let observed = self
            .first_height
            .collect_one(Day1::from(ranked_days))
            .map_or(start, usize::from)
            .min(start);
        let mut calibration = self
            .calibration
            .take()
            .filter(|c| c.end == observed && c.version == source_version)
            .unwrap_or_else(|| {
                Calibration::from_sources(
                    raw_loss_share,
                    cointime_loss_share,
                    coinflow_loss_share,
                    observed,
                    source_version,
                )
            });
        // Thresholds follow the stored loss shares alone, so one sequential pass fixes them
        // and the URPD evaluation can walk in parallel.
        // The ranks from the block before, already observed: the day it closes is rewritten.
        let ranked_from = observed.saturating_sub(1);
        let mut ranks = Vec::with_capacity(end - ranked_from);
        let thresholds = {
            let mut raw = raw_loss_share.bounded_cursor_at(ranked_from, end);
            let mut cointime = cointime_loss_share.bounded_cursor_at(ranked_from, end);
            let mut coinflow = coinflow_loss_share.bounded_cursor_at(ranked_from, end);
            (ranked_from..end)
                .filter_map(|index| {
                    let shares = Calibration::loss_shares(
                        raw.get(index).map(f64::from),
                        cointime.get(index).map(f64::from),
                        coinflow.get(index).map(f64::from),
                    );
                    let thresholds = (index >= start).then(|| calibration.thresholds(&shares));
                    if index >= observed {
                        calibration.observe(shares);
                    }
                    ranks.push(calibration.ranks(&shares));
                    thresholds
                })
                .collect::<Vec<_>>()
        };
        self.write_ranks(observed, end, &ranks, rank_version, context.exit())?;
        let mut last_progress = Instant::now();
        let mut walk = mem::take(&mut self.walk);
        walk.map(
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
            |worker, height, cells| {
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
                        cells,
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
                        self.modes.iter_mut().flat_map(ModeVecs::walk_stored_mut),
                        context.exit(),
                    )?;
                }
                Ok(())
            },
        )?;
        self.calibration = Some(calibration);
        self.walk = walk;
        self.compute_entry_supply(start, history, prices, context)
    }
}

impl Vecs {
    /// Each day's rank at its last block, once the blocks from `changed` on may have changed:
    /// `ranks` end before block `end` and reach back to the block before `changed`.
    fn write_ranks(
        &mut self,
        changed: usize,
        end: usize,
        ranks: &[Modes<f64>],
        version: Version,
        exit: &Exit,
    ) -> Result<()> {
        let Some(last) = end.checked_sub(1) else {
            return Ok(());
        };
        let last_day = self.day_of_height.collect_one_at(last).data()?.to_usize();
        let first_day = prepare_days(
            rank_vecs(&mut self.modes),
            version,
            &self.day_of_height,
            Height::from(changed),
            last_day + 1,
            exit,
        )?;
        let closes = day_closes(&self.first_height, first_day..last_day + 1, end);
        let from = end - ranks.len();
        debug_assert!(
            closes.iter().flatten().all(|&close| close >= from),
            "the ranks resume within the days they hold"
        );
        let none = Modes::from_fn(|_| f64::NAN);
        for close in closes {
            let values = close.map_or(&none, |height| &ranks[height - from]);
            for (vecs, &value) in self.modes.iter_mut().zip(values.iter()) {
                vecs.supply_in_profit_share_rank.push(Ratio::from(value));
            }
        }
        write_locked(rank_vecs(&mut self.modes), exit)
    }

    /// The floor bands' holdings, from the floors the walk stored: their own group, so each
    /// resumes from where it was written.
    fn compute_entry_supply(
        &mut self,
        max_from: usize,
        history: &Reader<'_>,
        prices: &dyn ReadableVec<Height, Cents>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let floors = Inputs::new(
            Version::ZERO,
            self.modes
                .iter()
                .flat_map(|vecs| vecs.prices_stored.floor.iter())
                .map(|vec| vec as &dyn AnyVec)
                .chain([prices as &dyn AnyVec]),
        );
        let range = floors.prepare(
            self.modes
                .iter_mut()
                .flat_map(|vecs| vecs.floor.iter_any_stored_mut())
                .collect::<Vec<_>>(),
            max_from,
            context.exit(),
        )?;
        let mut entry_supply = mem::take(&mut self.entry_supply);
        entry_supply.resume(&range, &self.modes, history, prices)?;
        entry_supply.compute(range, history, &mut self.modes, context.exit())?;
        self.entry_supply = entry_supply;
        Ok(())
    }
}

fn rank_vecs(modes: &mut Modes<ModeVecs>) -> Vec<&mut PcoVec<Day1, Ratio>> {
    modes
        .iter_mut()
        .map(|vecs| &mut vecs.supply_in_profit_share_rank)
        .collect()
}

/// One segment's cursors and cumulative-bucket scratch.
struct Worker<S, T, F> {
    supplies: AgeRange<S>,
    cointime: AgeRange<T>,
    coinflow: AgeRange<F>,
    scratch: Vec<CumulativeBucket>,
}
