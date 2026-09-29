use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use brk_types::Height;
use rayon::join;
use vecdb::AnyVec;

use super::{Vecs, value};
use crate::Dependencies;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn compute(
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

        self.db.sync_bg_tasks()?;

        let Vecs {
            value,
            origins,
            count,
            by_type,
            ..
        } = self;
        let (value_result, rest_result) = join(
            || {
                value::compute(
                    value,
                    origins,
                    indexer,
                    mappings,
                    indexer.safe_lengths().height,
                    Height::from(indexer.vecs().blocks.blockhash.len()),
                    exit,
                )
            },
            || {
                count.compute(indexer, blocks, exit)?;
                by_type.compute(indexer, exit)
            },
        );
        value_result?;
        rest_result?;

        context.compact_database(&self.db);
        Ok(())
    }
}
