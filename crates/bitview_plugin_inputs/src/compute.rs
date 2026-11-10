use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use brk_error::Result;
use brk_types::Height;
use rayon::join;
use vecdb::AnyVec;

use super::Vecs;
use crate::{
    Dependencies,
    counts::{Counts, compute_coinbase_witness},
};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies { indexer, mappings } = dependencies;
        let exit = context.exit();

        let Vecs {
            spends,
            count,
            types,
            counts,
            ..
        } = self;
        let Counts {
            ages,
            revealed,
            coinbase_witness,
        } = counts;
        let (spends_result, rest_result) = join(
            || {
                crate::spends::compute(
                    spends,
                    indexer,
                    mappings,
                    indexer.compute_start().height,
                    Height::from(indexer.vecs().blocks.blockhash.len()),
                    ages,
                    exit,
                )
            },
            || {
                count.compute(
                    indexer.compute_start().height,
                    &mappings.lookback.window_starts(),
                    exit,
                )?;
                types.compute(indexer, exit)?;
                revealed.compute(
                    indexer.compute_start().height,
                    &indexer.vecs().transactions.counters,
                    exit,
                )?;
                compute_coinbase_witness(
                    coinbase_witness,
                    indexer,
                    indexer.compute_start().height,
                    exit,
                )
            },
        );
        spends_result?;
        rest_result?;

        Ok(())
    }
}
