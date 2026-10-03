use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use rayon::join;
use vecdb::Database;

use super::{Vecs, count, features, fees, inscription, patterns, policy, sigops, size, versions};
use crate::Dependencies;

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
            inputs,
            mappings,
            blocks,
            price: prices,
        } = dependencies;
        let exit = context.exit();

        let ((r1, r2), (r3, r4)) = join(
            || {
                join(
                    || count::compute(&mut self.count, indexer, &blocks.lookback, exit),
                    || features::compute(&mut self.features, indexer, exit),
                )
            },
            || {
                join(
                    || versions::compute(&mut self.versions, indexer, exit),
                    || size::compute(&mut self.size, indexer, mappings, exit),
                )
            },
        );
        r1?;
        r2?;
        r3?;
        r4?;

        sigops::compute(&mut self.sigops, indexer, mappings, exit)?;

        fees::compute(
            &mut self.fees,
            indexer,
            &inputs.value,
            mappings,
            &self.size,
            &mut self.volume.transfer_volume.cumulative.sats.height,
            exit,
        )?;

        inscription::compute(&mut self.inscription, indexer, mappings, &self.fees, exit)?;

        patterns::compute(&mut self.patterns, indexer, &inputs.value, mappings, exit)?;

        policy::compute(&mut self.policy, indexer, mappings, &self.fees, exit)?;

        self.volume.transfer_volume.compute_cents(
            indexer.safe_lengths().height,
            &prices.spot.cents.height,
            exit,
        )?;

        Ok(())
    }
}
