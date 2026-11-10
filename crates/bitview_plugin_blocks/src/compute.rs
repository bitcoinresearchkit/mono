use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_primitives::{Count, Work};
use brk_error::Result;
use brk_types::SigOps;
use rayon::join;

use super::Vecs;
use crate::Dependencies;

/// The hashes expected to find a block of difficulty 1: 2^48 / 0xFFFF.
const HASHES_PER_DIFFICULTY: f64 = 281_474_976_710_656.0 / 65_535.0;

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
            empty_count,
            work,
            version_bit_4,
            coinbase_locktime,
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
        weight?;

        empty_count.compute_cumulative_transformed(
            starting_height,
            &mappings.height.tx_index_count,
            |txs| Count::from(u64::from(txs) == 1),
            exit,
        )?;
        crate::counts::compute_version_bit_4(version_bit_4, indexer, starting_height, exit)?;
        coinbase_locktime.compute(indexer, starting_height, exit)?;
        work.compute_cumulative_transformed(
            starting_height,
            &blocks.difficulty,
            |difficulty| Work::from(f64::from(difficulty) * HASHES_PER_DIFFICULTY),
            exit,
        )?;
        Ok(())
    }
}
