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
            op_return_value,
            unspendable_value,
            coinbase,
            pubkeys,
            ..
        } = self;
        let exit = context.exit();
        let start = indexer.compute_start().height;

        // Each walk over every output in place (see `dispatch::spread`).
        spent.compute(indexer, mappings, exit)?;
        let outputs = &indexer.vecs().outputs;
        // Price-independent output facts; require neither Inputs nor transaction analysis.
        value::compute_sats(
            value::Targets {
                op_return: &mut op_return_value.cumulative.sats.height,
                unspendable_value,
                values: &mut types.values,
                coinbase,
            },
            creations,
            usize::from(start)..outputs.first_txout_index.len(),
            indexer,
            exit,
        )?;
        db.flush();
        count.compute(start, &mappings.lookback.window_starts(), exit)?;
        let (counts, fiat) = join(
            || types.compute(indexer, exit),
            || op_return_value.compute_cents(start, &price.spot.cents.height, exit),
        );
        counts?;
        fiat?;
        for output in types.values.iter_mut() {
            output
                .value
                .compute_cents(start, &price.spot.cents.height, exit)?;
        }
        pubkeys.compute(start, &indexer.vecs().transactions.counters, exit)
    }
}
