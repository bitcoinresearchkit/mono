use brk_error::Result;

use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_indexer::Indexer;
use bitview_plugin_transactions::FeesVecs;
use brk_exit::Exit;
use vecdb::AnyVec;

use super::{Vecs, batch::Batch};
use crate::Dependencies;

const WRITE_INTERVAL: usize = 10_000;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<Self::Output> {
        self.compute_inner(dependencies.indexer, dependencies.fees, context.exit())
    }
}

impl Vecs {
    fn compute_inner(&mut self, indexer: &Indexer, fees: &FeesVecs, exit: &Exit) -> Result<()> {
        self.db.sync_bg_tasks()?;

        let starting_lengths = indexer.safe_lengths();
        let vecs = indexer.vecs();
        let raw = &vecs.op_return;
        let txs = &vecs.transactions;
        let version = raw.first_index.version()
            + raw.to_tx_index.version()
            + raw.kind.version()
            + raw.post_op_return_bytes.version()
            + txs.weight.version()
            + fees.fee.tx_index.version();

        self.validate_and_truncate(version, starting_lengths.height)?;

        let skip = self.min_len();
        let end = raw.first_index.len();
        if skip < end {
            self.truncate_if_needed_at(skip)?;

            for batch_start in (skip..end).step_by(WRITE_INTERVAL) {
                let batch_end = (batch_start + WRITE_INTERVAL).min(end);
                Batch::collect(indexer, fees, batch_start..batch_end).push_into(self);

                let _lock = exit.lock();
                self.write()?;
            }
        }

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });
        Ok(())
    }
}
