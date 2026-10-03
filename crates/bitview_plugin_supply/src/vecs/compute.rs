use brk_error::Result;

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
        let Dependencies {
            indexer,
            outputs,
            mining,
            price: prices,
        } = dependencies;
        let exit = context.exit();

        self.burned
            .compute(indexer, outputs, mining, prices, exit)?;

        Ok(())
    }
}
