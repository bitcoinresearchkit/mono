use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use brk_error::Result;

use crate::{Dependencies, Vecs};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

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
