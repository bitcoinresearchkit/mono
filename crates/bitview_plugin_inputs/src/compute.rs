use bitview_plugin::{ComputePlugin, DependentContext, compute_items};
use brk_error::Result;
use brk_types::Height;
use rayon::join;
use vecdb::AnyVec;

use super::Vecs;
use crate::Dependencies;

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: DependentContext<'_>,
    ) -> Result<()> {
        let starting_height = context.start().height;
        let Dependencies { indexer, mappings } = dependencies;
        let exit = context.exit();

        let Vecs {
            spends,
            count,
            types,
            ..
        } = self;
        let (spends_result, rest_result) = join(
            || {
                crate::spends::compute(
                    spends,
                    indexer,
                    mappings,
                    starting_height,
                    Height::from(indexer.vecs().blocks.blockhash.len()),
                    exit,
                )
            },
            || {
                count.compute(
                    starting_height,
                    &mappings.lookback.window_starts(),
                    exit,
                )?;
                types.compute(indexer, starting_height, exit)
            },
        );
        spends_result?;
        rest_result?;

        Ok(())
    }
}
