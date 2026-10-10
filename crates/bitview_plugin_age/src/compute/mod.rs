mod context;
mod origin_loop;
pub(crate) mod origin_targets;
mod price_range_max;
pub use context::ComputeContext;
pub(crate) use origin_loop::replay_origins;
pub use price_range_max::PriceRangeMax;

use bitview_compute::prepare_computed;
use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_types::{Height, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, Database, ReadableVec, Stamp};

use crate::{Dependencies, Vecs, live::LiveState, state::UTXOStates};
use origin_targets::OriginTargets;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }

    fn compute_state(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        // A version reset must invalidate the resident state even if validation fails.
        let live = self.live.take();
        let exit = context.exit();
        let (spends, creations) = deps.history.versions();
        let base_version = Version::ONE
            + Version::from(spends as u32)
            + Version::from(creations as u32)
            + deps.price.spot.cents.height.version()
            + deps.mappings.timestamp.monotonic.version();
        let end = deps
            .history
            .len()
            .min(deps.price.spot.cents.height.len())
            .min(deps.mappings.timestamp.monotonic.len());
        let start = prepare_computed(
            self.outputs_mut(),
            base_version,
            usize::from(deps.from).min(end),
            exit,
        )?;
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
        let mut from = start;
        while from < end {
            let next = (from + 10_000).min(end);
            ctx.starting_height = Height::from(from);
            ctx.last_height = Height::from(next - 1);
            replay_origins(
                &mut OriginTargets {
                    ranges: &mut self.ranges,
                    epochs: &mut self.epochs,
                    classes: &mut self.classes,
                    coinblocks_destroyed: &mut self.coinblocks_destroyed,
                    all_capital: &mut self.all_capital,
                },
                &mut states,
                &ctx,
                &mut cursor,
            )?;
            self.save(cursor.state().len(), next == end, exit)?;
            from = next;
        }
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
    /// Every output the replay writes: one dependency version, one resume height.
    fn outputs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs: Vec<&mut dyn AnyStoredVec> = Vec::with_capacity(1024);
        for range in self.ranges.iter_mut() {
            vecs.extend(range.stored_vecs_mut());
        }
        for cohort in self.epochs.iter_mut().chain(self.classes.iter_mut()) {
            vecs.extend(cohort.stored_vecs_mut());
        }
        vecs.push(self.coinblocks_destroyed.stored_mut());
        vecs.push(&mut self.all_capital);
        vecs
    }

    fn save(&mut self, end: usize, final_chunk: bool, exit: &Exit) -> Result<()> {
        let _lock = exit.lock();
        let stamp = Stamp::from(Height::from(end - 1));
        self.outputs_mut()
            .into_par_iter()
            .try_for_each(|v| v.any_stamped_write_maybe_with_changes(stamp, final_chunk))?;
        self.db.flush();
        Ok(())
    }
}
