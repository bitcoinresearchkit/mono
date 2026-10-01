use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_types::{Height, Lengths, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, Stamp};

use crate::{
    Dependencies,
    compute::{ComputeContext, PriceRangeMax, origin_targets::OriginTargets, replay_origins},
    live::LiveState,
    state::UTXOStates,
};

use super::Vecs;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn compute(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        // A version reset must invalidate the resident state even if validation fails.
        let live = self.live.take();
        self.db.sync_bg_tasks()?;
        let exit = context.exit();
        let (spends, creations) = deps.history.versions();
        let base_version = Version::ONE
            + Version::from(spends as u32)
            + Version::from(creations as u32)
            + deps.price.spot.cents.height.version()
            + deps.mappings.timestamp.monotonic.version();
        {
            let _lock = exit.lock();
            for v in self.coindays_created.iter_mut() {
                v.validate_computed_version_or_reset(base_version)?;
            }
            self.coinblocks_destroyed
                .validate_computed_version_or_reset(base_version)?;
            self.cohorts.validate_computed_versions(base_version)?;
        }
        let end = deps
            .history
            .len()
            .min(deps.price.spot.cents.height.len())
            .min(deps.mappings.timestamp.monotonic.len());
        let start = usize::from(deps.from)
            .min(end)
            .min(usize::from(self.cohorts.min_resume_len()))
            .min(self.age_bounds.len())
            .min(self.coinblocks_destroyed.block.len())
            .min(
                self.coindays_created
                    .iter()
                    .map(|v| v.cumulative.height.len())
                    .min()
                    .unwrap_or_default(),
            );
        let Dependencies {
            history,
            mappings,
            price: prices,
            ..
        } = deps;
        if end == 0 {
            return Ok(());
        }
        let version = (
            prices.spot.cents.height.version(),
            mappings.timestamp.monotonic.version(),
            history.versions(),
        );
        let live = match live {
            Some(live)
                if live.origins.len() == start
                    && live.version == version
                    && history.matches(&live.origins)? =>
            {
                Some(live)
            }
            _ => None,
        };
        let reuse = live.is_some();
        let LiveState {
            mut origins,
            mut states,
            prices: mut price_data,
            mut timestamps,
            mut max,
            ..
        } = if let Some(live) = live {
            live
        } else {
            let price_data = prices.spot.cents.height.collect_range_at(0, start);
            LiveState {
                origins: history.state_at(start)?,
                states: UTXOStates::new(),
                prices: price_data,
                timestamps: mappings.timestamp.monotonic.collect_range_at(0, start),
                max: PriceRangeMax::default(),
                version,
            }
        };
        price_data.extend(prices.spot.cents.height.collect_range_at(start, end));
        timestamps.extend(mappings.timestamp.monotonic.collect_range_at(start, end));
        if price_data.len() != end || timestamps.len() != end {
            return Err(Error::NotFound(
                "incomplete age price or timestamp history".into(),
            ));
        }
        max.extend(&price_data);
        let mut ctx = ComputeContext {
            starting_height: Height::from(start),
            last_height: Height::from(end - 1),
            height_to_timestamp: &timestamps,
            height_to_price: &price_data,
            price_range_max: &max,
        };
        if !reuse {
            states.restore_origins(origins.amounts(), &ctx)?;
        }
        let mut cursor = history.cursor(&mut origins)?;
        {
            let _lock = exit.lock();
            self.par_iter_state_vecs_mut()
                .try_for_each(|v| v.any_truncate_if_needed_at(start))?;
            for v in self.age_bounds.stored_vecs_mut() {
                v.any_truncate_if_needed_at(start)?;
            }
        }
        let mut from = start;
        while from < end {
            let next = (from + 10_000).min(end);
            ctx.starting_height = Height::from(from);
            ctx.last_height = Height::from(next - 1);
            replay_origins(
                &mut OriginTargets {
                    cohorts: &mut self.cohorts,
                    coindays_created: &mut self.coindays_created,
                    coinblocks_destroyed: &mut self.coinblocks_destroyed,
                },
                &mut states,
                &ctx,
                &mut cursor,
                |height, states| self.age_bounds.push_block(height, states.bounds_entries()),
            )?;
            self.save(cursor.state().len(), next == end, exit)?;
            from = next;
        }
        let lengths = Lengths {
            height: Height::from(start),
            ..Default::default()
        };
        self.cohorts.compute_rest_part1(&lengths, exit)?;
        self.cohorts.compute_rest_part2(&lengths, exit)?;
        context.compact_database(&self.db);
        self.live = Some(LiveState {
            origins,
            states,
            prices: price_data,
            timestamps,
            max,
            version,
        });
        Ok(())
    }
}

impl Vecs {
    fn par_iter_state_vecs_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.cohorts
            .par_iter_vecs_mut()
            .chain(
                self.coindays_created
                    .iter_mut()
                    .map(|v| v.stored_mut())
                    .collect::<Vec<_>>()
                    .into_par_iter(),
            )
            .chain([self.coinblocks_destroyed.stored_mut()].into_par_iter())
    }

    fn save(&mut self, end: usize, final_chunk: bool, exit: &Exit) -> Result<()> {
        let _lock = exit.lock();
        let stamp = Stamp::from(Height::from(end - 1));
        self.par_iter_state_vecs_mut()
            .try_for_each(|v| v.any_stamped_write_maybe_with_changes(stamp, final_chunk))?;
        self.age_bounds.write()?;
        self.db.flush()?;
        Ok(())
    }
}
