use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use brk_types::Height;
use rayon::join;
use vecdb::{AnyVec, Database};

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
            blocks,
            mappings,
        } = dependencies;
        let exit = context.exit();

        let Vecs {
            origins,
            count,
            types,
            ..
        } = self;
        let (origins_result, rest_result) = join(
            || {
                origins.compute(
                    indexer,
                    mappings,
                    indexer.safe_lengths().height,
                    Height::from(indexer.vecs().blocks.blockhash.len()),
                    exit,
                )
            },
            || {
                count.compute(indexer, blocks, exit)?;
                types.compute(indexer, exit)
            },
        );
        origins_result?;
        rest_result?;

        Ok(())
    }
}
