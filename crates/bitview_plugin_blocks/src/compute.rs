use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use brk_error::Result;
use brk_types::SigOps;
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
        let Dependencies { indexer, mappings } = dependencies;
        let exit = context.exit();
        let starting_height = indexer.compute_start().height;
        let window_starts = mappings.lookback.window_starts();
        let blocks = &indexer.vecs().blocks;

        // Interval, size, weight and sigop cost are independent.
        let Vecs {
            interval,
            size,
            weight,
            sigop_cost,
            ..
        } = self;
        let ((interval, sigop_cost), (size, weight)) = join(
            || {
                join(
                    || interval.compute(indexer, exit),
                    || {
                        sigop_cost.compute_cumulative_sum_from_indexes(
                            starting_height,
                            &indexer.vecs().transactions.first_tx_index,
                            &mappings.height.tx_index_count,
                            &*indexer.vecs().transactions.total_sigop_cost,
                            SigOps::from,
                            exit,
                        )
                    },
                )
            },
            || {
                join(
                    || size.compute(starting_height, &window_starts, &*blocks.size, exit),
                    || weight.compute(starting_height, &window_starts, &*blocks.weight, exit),
                )
            },
        );
        interval?;
        sigop_cost?;
        size?;
        weight
    }
}
