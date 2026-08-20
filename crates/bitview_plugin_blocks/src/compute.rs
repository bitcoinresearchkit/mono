use brk_error::Result;

use std::thread;

use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_indexer::Indexer;
use vecdb::Exit;

use super::{Vecs, interval::Compute as _, lookback::Invalidate as _, size::Compute as _};
use crate::Dependencies;

impl Vecs {
    fn compute_inner(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        self.db.sync_bg_tasks()?;

        // Cached lookbacks depend on the monotonic timestamp vec, which may
        // have changed without changing its final length after a reorg.
        self.lookback.invalidate_caches();

        // Interval and size are independent.
        let Vecs {
            lookback,
            interval,
            size,
            ..
        } = self;
        thread::scope(|s| -> Result<()> {
            let r1 = s.spawn(|| interval.compute(indexer, exit));
            size.compute(indexer, &*lookback, exit)?;
            r1.join().unwrap()?;
            Ok(())
        })?;

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });
        Ok(())
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<Self::Output> {
        self.compute_inner(dependencies.indexer, context.exit())
    }
}
