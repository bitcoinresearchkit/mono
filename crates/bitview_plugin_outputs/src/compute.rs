use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_indexer::Indexer;
use brk_error::Result;
use brk_exit::Exit;
use rayon::join;
use vecdb::{AnyVec, Database};

use super::{Vecs, value};
use crate::Dependencies;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        None
    }

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies {
            indexer,
            blocks,
            price,
        } = dependencies;
        self.compute_created(
            indexer,
            usize::from(indexer.safe_lengths().height),
            indexer.vecs().outputs.first_txout_index.len(),
            context.exit(),
        )?;
        let Vecs {
            db,
            value,
            count,
            by_type,
            spent,
            ..
        } = self;

        let exit = context.exit();
        count.compute(indexer, blocks, exit)?;
        let (fiat, types) = join(
            || {
                value.op_return.compute_cents(
                    indexer.safe_lengths().height,
                    &price.spot.cents.height,
                    exit,
                )
            },
            || by_type.compute(indexer, exit),
        );
        fiat?;
        types?;
        let lock = spent.compute(indexer, exit)?;
        db.run_bg(move |db| {
            let _lock = lock;
            db.compact_deferred_default()
        });
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
        self.db.sync_bg_tasks()?;
        let outputs = &indexer.vecs().outputs;
        value::compute_sats(
            &mut self.value.op_return.cumulative.sats.height,
            &mut self.creations,
            start..end,
            &outputs.first_txout_index,
            (&outputs.output_type, &outputs.value),
            &indexer.vecs().blocks.blockhash,
            exit,
        )?;
        self.db.flush()?;
        Ok(())
    }
}
