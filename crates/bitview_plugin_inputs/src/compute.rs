use brk_error::Result;

use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_blocks::Vecs as BlockVecs;
use bitview_plugin_indexer::Indexer;
use rayon::join;
use vecdb::Exit;

use super::Vecs;
use crate::Dependencies;

impl Vecs {
    fn compute_inner(&mut self, indexer: &Indexer, blocks: &BlockVecs, exit: &Exit) -> Result<()> {
        self.db.sync_bg_tasks()?;

        let Vecs {
            value,
            count,
            by_type,
            ..
        } = self;
        let (value_result, rest_result) = join(
            || super::value::compute(value, indexer, exit),
            || {
                count.compute(indexer, blocks, exit)?;
                by_type.compute(indexer, exit)
            },
        );
        value_result?;
        rest_result?;

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
        self.compute_inner(dependencies.indexer, dependencies.blocks, context.exit())
    }
}
