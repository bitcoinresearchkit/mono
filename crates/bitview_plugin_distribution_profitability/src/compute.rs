use bitview_compute::prepare_computed;
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_distribution_common::state::cost_basis::PriceIndex;
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_types::{CentsCompact, Height, Version};
use vecdb::{AnyVec, Database, ReadableVec, Stamp};

use crate::{
    Dependencies, Vecs,
    live::{LiveState, advance, ranges},
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
        let (mut live, reuse) = match live {
            Some(live)
                if live.origins.len() == start
                    && live.version == version
                    && deps.history.matches(&live.origins)? =>
            {
                (live, true)
            }
            _ => (
                LiveState {
                    origins: deps.history.state_at(start)?,
                    index: PriceIndex::default(),
                    prices: Vec::new(),
                    timestamps: Vec::new(),
                    crossings: [0; 3],
                    version,
                },
                false,
            ),
        };
        let prices = deps.prices.collect_range_at(live.prices.len(), end);
        if prices.iter().any(|p| p.is_nan()) {
            return Err(Error::NotFound(
                "invalid profitability price history".into(),
            ));
        }
        live.prices
            .extend(prices.iter().copied().map(CentsCompact::from));
        live.timestamps
            .extend(deps.timestamps.collect_range_at(live.timestamps.len(), end));
        if live.prices.len() != end || live.timestamps.len() != end {
            return Err(Error::NotFound(
                "incomplete profitability price or timestamp history".into(),
            ));
        }
        if !reuse {
            live.restore();
        }
        let price_start = if reuse { start } else { 0 };
        let mut cursor = deps.history.cursor(&mut live.origins)?;
        for h in start..end {
            let spot = prices[h - price_start];
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
