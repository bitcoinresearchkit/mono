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
        let starting_lengths = context.start();
        let Dependencies {
            price: prices,
            mappings,
        } = dependencies;
        let exit = context.exit();

        let (ath, (range, moving_average)) = join(
            || {
                self.ath
                    .compute(prices, mappings, starting_lengths.height, exit)
            },
            || {
                join(
                    || {
                        self.range
                            .compute(prices, &mappings.lookback, starting_lengths.height, exit)
                    },
                    || {
                        self.moving_average
                            .compute(&mappings.lookback, prices, starting_lengths, exit)
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
