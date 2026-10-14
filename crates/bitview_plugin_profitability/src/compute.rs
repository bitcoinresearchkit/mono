use bitview_compute::{Inputs, save_stamped, write_chunks};
use bitview_distribution::state::cost_basis::{PRICE_INDEX_VERSION, age_index};
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use vecdb::{AnyVec, Database};

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
        let exit = context.exit();
        let mut inputs = Inputs::new(
            Version::TWO + PRICE_INDEX_VERSION + age_index::history_version(deps.history),
            [deps.prices as &dyn AnyVec, deps.timestamps],
        );
        inputs.len = inputs.len.min(deps.history.len());
        let version = inputs.version;
        let range = inputs.prepare(
            self.metrics.iter_any_stored_mut().collect::<Vec<_>>(),
            usize::from(deps.from),
            exit,
        )?;
        let (start, end) = (range.start, range.end);
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
        for chunk in write_chunks(range) {
            for &spot in &spots[chunk.start - start..chunk.end - start] {
                age_index::advance(
                    &mut live.index,
                    &mut cursor,
                    &live.prices,
                    &live.timestamps,
                    &mut live.crossings,
                )?;
                let (bands, totals) = ranges(&live.index, spot);
                self.metrics.push(&bands, &totals);
            }
            save_stamped(
                &self.db,
                self.metrics.iter_any_stored_mut(),
                chunk.end,
                true,
                exit,
            )?;
        }
        drop(cursor);
        self.live = Some(live);
        Ok(())
    }
}
