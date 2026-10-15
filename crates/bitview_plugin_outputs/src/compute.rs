use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use brk_error::Result;
use rayon::join;
use vecdb::AnyVec;

use super::{Vecs, value};
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
            mappings,
            price,
        } = dependencies;
        let Vecs {
            db,
            creations,
            spent,
            count,
            types,
            value,
            ..
        } = self;
        let exit = context.exit();
        let start = indexer.compute_start().height;

        // The spend index reads only indexer columns: it runs beside everything else.
        let (spent, created) = join(
            || spent.compute(indexer, exit),
            || -> Result<()> {
                let outputs = &indexer.vecs().outputs;
                // Price-independent output facts; require neither Inputs nor transaction
                // analysis.
                value::compute_sats(
                    &mut value.op_return_value.cumulative.sats.height,
                    creations,
                    usize::from(start)..outputs.first_txout_index.len(),
                    &outputs.first_txout_index,
                    (&outputs.output_type, &outputs.value),
                    &indexer.vecs().blocks.blockhash,
                    exit,
                )?;
                db.flush();
                count.compute(start, &mappings.lookback.window_starts(), exit)?;
                let (fiat, counts) = join(
                    || {
                        value
                            .op_return_value
                            .compute_cents(start, &price.spot.cents.height, exit)
                    },
                    || types.compute(indexer, exit),
                );
                fiat?;
                counts
            },
        );
        spent?;
        created
    }
}
