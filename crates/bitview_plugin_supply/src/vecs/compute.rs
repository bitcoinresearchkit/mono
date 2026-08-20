use brk_error::Result;

use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;
use crate::{Dependencies, burned};

impl Vecs {
    fn compute_inner(
        &mut self,
        indexer: &Indexer,
        outputs: &bitview_plugin_outputs::Vecs,
        mining: &bitview_plugin_mining::Vecs,
        prices: &bitview_plugin_price::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        // 1. Compute burned/unspendable supply
        burned::compute(&mut self.burned, indexer, outputs, mining, prices, exit)?;

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
        self.compute_inner(
            dependencies.indexer,
            dependencies.outputs,
            dependencies.mining,
            dependencies.price,
            context.exit(),
        )
    }
}
