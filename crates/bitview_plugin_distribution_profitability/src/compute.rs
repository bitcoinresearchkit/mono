use bitview_compute::prepare_computed;
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_distribution_common::state::cost_basis::age_index::advance;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use vecdb::{AnyVec, Database, Stamp};

use crate::{
    Dependencies, Vecs,
    live::{LiveState, ranges},
};

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }

    fn compute_state(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        let live = self.live.take();
        let version = (
            deps.prices.version(),
            deps.timestamps.version(),
            deps.history.versions(),
        );
        let base_version = Version::ONE
            + version.0
            + version.1
            + Version::from(version.2.0 as u32)
            + Version::from(version.2.1 as u32);
        let end = deps
            .history
            .len()
            .min(deps.prices.len())
            .min(deps.timestamps.len());
        let start = prepare_computed(
            self.metrics.stored_vecs_mut().collect::<Vec<_>>(),
            base_version,
            usize::from(deps.from).min(end),
            context.exit(),
        )?;
        if end == 0 {
            return Ok(());
        }
        let (mut live, spots) = LiveState::resume(
            live,
            version,
            deps.history,
            start,
            end,
            deps.prices,
            deps.timestamps,
        )?;
        let mut cursor = deps.history.cursor(&mut live.origins)?;
        for (h, &spot) in (start..end).zip(&spots) {
            advance(
                &mut live.index,
                &mut cursor,
                &live.prices,
                &live.timestamps,
                &mut live.crossings,
            )?;
            self.metrics.push(spot, &ranges(&live.index, spot));
            if (h + 1).is_multiple_of(10_000) || h + 1 == end {
                self.save(h + 1, context.exit())?;
            }
        }
        drop(cursor);
        self.live = Some(live);
        Ok(())
    }
}

impl Vecs {
    fn save(&mut self, end: usize, exit: &Exit) -> Result<()> {
        let _lock = exit.lock();
        let stamp = Stamp::from(Height::from(end - 1));
        for v in self.metrics.stored_vecs_mut() {
            v.any_stamped_write_maybe_with_changes(stamp, true)?;
        }
        self.db.flush()?;
        Ok(())
    }
}
