use bitview_compute::{prepare_computed, write_chunks, write_locked};
use bitview_plugin::{ComputePlugin, DependentContext, compute_items};
use brk_error::Result;
use vecdb::{AnyVec, VecIndex};

use crate::{Dependencies, Vecs, batch::Batch};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: DependentContext<'_>,
    ) -> Result<()> {
        let starting_lengths = context.start();
        let Dependencies {
            indexer,
            fee,
            spot_cents,
        } = dependencies;
        let exit = context.exit();

        let vecs = indexer.vecs();
        let raw = &vecs.op_return;
        let txs = &vecs.transactions;
        let version = raw.first_index.version()
            + raw.to_tx_index.version()
            + raw.kind.version()
            + raw.post_op_return_bytes.version()
            + txs.first_tx_index.version()
            + txs.weight.version()
            + fee.version();

        let skip = prepare_computed(
            self.stored_mut().collect::<Vec<_>>(),
            version,
            starting_lengths.height.to_usize(),
            exit,
        )?;
        let end = raw.first_index.len();
        for chunk in write_chunks(skip..end) {
            Batch::collect(indexer, fee, chunk).push_into(self);
            write_locked(self.stored_mut(), exit)?;
        }

        for bucket in self.buckets_mut() {
            bucket
                .fees
                .fees
                .compute_cents(starting_lengths.height, spot_cents, exit)?;
        }
        Ok(())
    }
}
