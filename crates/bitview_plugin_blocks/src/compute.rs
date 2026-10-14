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
        let Dependencies { indexer, mappings } = dependencies;
        let exit = context.exit();
        let starting_height = indexer.safe_lengths().height;
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
