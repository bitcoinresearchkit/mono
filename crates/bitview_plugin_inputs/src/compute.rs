use brk_error::Result;

use bitview_plugin::ComputePlugin;
use bitview_plugin_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;
impl Vecs {
    fn compute_inner(
        &mut self,
        indexer: &Indexer,
        blocks: &bitview_plugin_blocks::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        super::value::compute_value(self, indexer, exit)?;
        self.count.compute(indexer, blocks, exit)?;
        self.by_type.compute(indexer, exit)?;

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });
        Ok(())
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = crate::Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        exit: &Exit,
    ) -> Result<Self::Output> {
        self.compute_inner(dependencies.indexer, dependencies.blocks, exit)
    }
}
