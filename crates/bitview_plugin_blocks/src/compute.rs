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
        let Dependencies { indexer, mappings } = dependencies;
        let exit = context.exit();
        let starting_height = indexer.compute_start().height;
        let window_starts = mappings.lookback.window_starts();
        let blocks = &indexer.vecs().blocks;

        // Interval, size and weight are independent.
        let Vecs {
            interval,
            size,
            weight,
            ..
        } = self;
        let (interval, (size, weight)) = join(
            || interval.compute(indexer, exit),
            || {
                join(
                    || size.compute(starting_height, &window_starts, &*blocks.total, exit),
                    || weight.compute(starting_height, &window_starts, &*blocks.weight, exit),
                )
            },
        );
        interval?;
        size?;
        weight
    }
}
