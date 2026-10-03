use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use rayon::join;
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
            price: prices,
            mappings,
            blocks,
        } = dependencies;
        let exit = context.exit();

        let (ath, (range, moving_average)) = join(
            || self.ath.compute(indexer, prices, mappings, exit),
            || {
                join(
                    || self.range.compute(indexer, prices, blocks, exit),
                    || self.moving_average.compute(indexer, blocks, prices, exit),
                )
            },
        );
        ath?;
        range?;
        moving_average?;

        let (returns, technical) = join(
            || self.returns.compute(indexer, blocks, exit),
            || {
                self.technical
                    .compute(indexer, prices, blocks, &self.moving_average, exit)
            },
        );
        returns?;
        technical?;

        Ok(())
    }
}
