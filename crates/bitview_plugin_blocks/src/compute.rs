use brk_error::Result;

use std::thread;

use bitview_plugin::{ComputePlugin, UpdateContext};
use vecdb::Database;

use super::Vecs;
use crate::Dependencies;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies { indexer } = dependencies;
        let exit = context.exit();

        // Interval, size and weight are independent.
        let Vecs {
            lookback,
            interval,
            size,
            weight,
            ..
        } = self;
        thread::scope(|s| -> Result<()> {
            let interval = s.spawn(|| interval.compute(indexer, exit));
            let size = s.spawn(|| size.compute(indexer, &*lookback, exit));
            weight.compute(indexer, &*lookback, exit)?;
            interval.join().unwrap()?;
            size.join().unwrap()?;
            Ok(())
        })?;

        Ok(())
    }
}
