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

        Ok(())
    }
}
