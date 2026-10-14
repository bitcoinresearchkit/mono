mod context;
mod origin_loop;
pub(crate) mod origin_targets;
mod price_range_max;
pub use context::ComputeContext;
pub(crate) use origin_loop::replay_origins;
pub use price_range_max::PriceRangeMax;

use bitview_compute::{Inputs, write_chunks};
use bitview_distribution::state::cost_basis::age_index;
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_traversable::Traversable;
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_types::{Height, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, Database, ReadableVec, Stamp};

use crate::{Dependencies, Vecs, live::LiveState, state::UTXOStates};
use origin_targets::OriginTargets;

/// Every output the replay writes (1,144 at present).
const OUTPUT_CAPACITY: usize = 2_048;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }

    fn compute_state(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        // A version reset must invalidate the resident state even if validation fails.
        let live = self.live.take();
        let exit = context.exit();
        let Dependencies {
            history,
            from,
            mappings,
            price: prices,
        } = deps;
        let spot = &prices.spot.cents.height;
        let monotonic = &mappings.timestamp.monotonic;
        let mut inputs = Inputs::new(
            Version::ONE + age_index::history_version(history),
            [spot as &dyn AnyVec, monotonic],
        );
        inputs.len = inputs.len.min(history.len());
        let version = inputs.version;
        let range = inputs.prepare(self.outputs_mut(), usize::from(from), exit)?;
        let (start, end) = (range.start, range.end);
        if end == 0 {
            return Ok(());
        }
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
        } = match live {
            Some(live) => live,
            None => LiveState {
                origins: history.state_at(start)?,
                states: UTXOStates::new(),
                prices: Vec::new(),
                timestamps: Vec::new(),
                max: PriceRangeMax::default(),
                version,
            },
        };
        spot.read_into_at(price_data.len(), end, &mut price_data);
        monotonic.read_into_at(timestamps.len(), end, &mut timestamps);
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
            states.restore_origins(origins.amounts(), &ctx);
        }
        let mut cursor = history.cursor(&mut origins)?;
        for chunk in write_chunks(range) {
            ctx.starting_height = Height::from(chunk.start);
            ctx.last_height = Height::from(chunk.end - 1);
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
            self.save(chunk.end, chunk.end == end, exit)?;
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
        let mut vecs = Vec::with_capacity(OUTPUT_CAPACITY);
        vecs.extend(self.iter_any_stored_mut());
        vecs
    }

    /// Saves the outputs computed through `end`, stamped, in parallel (`save_stamped` writes one
    /// at a time); the final chunk keeps rollback changes.
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
