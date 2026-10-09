use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use vecdb::{AnyVec, Database};

use crate::{Dependencies, Vecs, batch::Batch};

const WRITE_INTERVAL: usize = 10_000;

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

        {
            let _lock = exit.lock();
            self.validate_and_truncate(version, starting_lengths.height)?;
        }

        let skip = self.min_len();
        let end = raw.first_index.len();
        if skip < end {
            {
                let _lock = exit.lock();
                self.truncate_if_needed_at(skip)?;
            }

            for batch_start in (skip..end).step_by(WRITE_INTERVAL) {
                let batch_end = (batch_start + WRITE_INTERVAL).min(end);
                Batch::collect(indexer, fees, batch_start..batch_end).push_into(self);

                let _lock = exit.lock();
                self.write()?;
            }
        }

        let prices = &price.spot.cents.height;
        let height = starting_lengths.height;
        self.total.compute_cents(height, prices, exit)?;
        self.protocols.compute_cents(height, prices, exit)?;
        self.policies.compute_cents(height, prices, exit)?;

        Ok(())
    }
}
