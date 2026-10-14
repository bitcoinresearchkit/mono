use bitview_compute::{prepare_computed, write_chunks, write_locked};
use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use vecdb::{AnyVec, Database, VecIndex};

use crate::{Dependencies, Vecs, batch::Batch};

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
        let Dependencies {
            indexer,
            fees,
            price,
        } = dependencies;
        let exit = context.exit();

        let starting_lengths = indexer.compute_start();
        let vecs = indexer.vecs();
        let raw = &vecs.op_return;
        let txs = &vecs.transactions;
        let version = raw.first_index.version()
            + raw.to_tx_index.version()
            + raw.kind.version()
            + raw.post_op_return_bytes.version()
            + txs.weight.version()
            + fees.fee.tx.version();

        let skip = prepare_computed(
            self.stored_mut().collect::<Vec<_>>(),
            version,
            starting_lengths.height.to_usize(),
            exit,
        )?;
        let end = raw.first_index.len();
        for chunk in write_chunks(skip..end) {
            Batch::collect(indexer, fees, chunk).push_into(self);
            write_locked(self.stored_mut(), exit)?;
        }

        let prices = &price.spot.cents.height;
        let height = starting_lengths.height;
        self.total.compute_cents(height, prices, exit)?;
        self.protocols.compute_cents(height, prices, exit)?;
        self.policies.compute_cents(height, prices, exit)?;

        Ok(())
    }
}
