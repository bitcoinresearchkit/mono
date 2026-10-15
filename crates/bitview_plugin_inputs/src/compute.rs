use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
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
        context: UpdateContext<'_>,
    ) -> Result<()> {
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
                    indexer.compute_start().height,
                    Height::from(indexer.vecs().blocks.blockhash.len()),
                    exit,
                )
            },
            || {
                count.compute(indexer, mappings, exit)?;
                types.compute(indexer, exit)
            },
        );
        spends_result?;
        rest_result?;

        Ok(())
    }
}
