use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use rayon::join;
use vecdb::Database;

use super::{Vecs, ath, moving_average, range, returns, technical};
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
            || ath::compute(&mut self.ath, indexer, prices, mappings, exit),
            || {
                join(
                    || range::compute(&mut self.range, indexer, prices, blocks, exit),
                    || {
                        moving_average::compute(
                            &mut self.moving_average,
                            indexer,
                            blocks,
                            prices,
                            exit,
                        )
                    },
                )
            },
        );
        ath?;
        range?;
        moving_average?;

        let (returns, technical) = join(
            || returns::compute(&mut self.returns, indexer, blocks, exit),
            || {
                technical::compute(
                    &mut self.technical,
                    indexer,
                    prices,
                    blocks,
                    &self.moving_average,
                    exit,
                )
            },
        );
        returns?;
        technical?;

        Ok(())
    }
}
