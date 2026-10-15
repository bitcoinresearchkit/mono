use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_plugin_indexer::Indexer;
use brk_error::Result;
use brk_exit::Exit;
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
        self.compute_created(
            indexer,
            usize::from(indexer.compute_start().height),
            indexer.vecs().outputs.first_txout_index.len(),
            context.exit(),
        )?;
        let Vecs {
            value,
            count,
            types,
            spent,
            ..
        } = self;

        let exit = context.exit();
        count.compute(indexer, mappings, exit)?;
        let (fiat, counts) = join(
            || {
                value.op_return_value.compute_cents(
                    indexer.compute_start().height,
                    &price.spot.cents.height,
                    exit,
                )
            },
            || types.compute(indexer, exit),
        );
        fiat?;
        counts?;
        spent.compute(indexer, exit)?;
        Ok(())
    }
}

impl Vecs {
    /// Price-independent output facts; requires neither Inputs nor transaction analysis.
    fn compute_created(
        &mut self,
        indexer: &Indexer,
        start: usize,
        end: usize,
        exit: &Exit,
    ) -> Result<()> {
        let outputs = &indexer.vecs().outputs;
        value::compute_sats(
            &mut self.value.op_return_value.cumulative.sats.height,
            &mut self.creations,
            start..end,
            &outputs.first_txout_index,
            (&outputs.output_type, &outputs.value),
            &indexer.vecs().blocks.blockhash,
            exit,
        )?;
        self.db.flush();
        Ok(())
    }
}
