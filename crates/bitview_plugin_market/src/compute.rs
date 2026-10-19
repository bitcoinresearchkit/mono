use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use brk_error::Result;
use rayon::join;

use super::Vecs;
use crate::Dependencies;

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies {
            indexer,
            price: prices,
            mappings,
        } = dependencies;
        let exit = context.exit();

        let (ath, (range, moving_average)) = join(
            || self.ath.compute(indexer, prices, mappings, exit),
            || {
                join(
                    || {
                        self.range
                            .compute(indexer, prices, &mappings.lookback, exit)
                    },
                    || {
                        self.moving_average
                            .compute(indexer, &mappings.lookback, prices, exit)
                    },
                )
            },
        );
        ath?;
        range?;
        moving_average?;
        Ok(())
    }
}
