use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use rayon::join;
use vecdb::Database;

use super::Vecs;
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
                    || self.count.compute(indexer, &blocks.lookback, exit),
                    || self.features.compute(indexer, exit),
                )
            },
            || {
                join(
                    || self.versions.compute(indexer, exit),
                    || self.size.compute(indexer, mappings, exit),
                )
            },
        );
        r1?;
        r2?;
        r3?;
        r4?;

        self.sigops.compute(indexer, mappings, exit)?;

        self.fees.compute(
            indexer,
            &inputs.value,
            mappings,
            &self.size,
            &mut self.volume.cumulative.sats.height,
            exit,
        )?;

        self.inscription
            .compute(indexer, mappings, &self.fees, exit)?;

        self.patterns
            .compute(indexer, &inputs.value, mappings, exit)?;

        self.policy.compute(indexer, mappings, &self.fees, exit)?;

        self.volume.compute_cents(
            indexer.safe_lengths().height,
            &prices.spot.cents.height,
            exit,
        )?;
        self.inscription.fees.compute_cents(
            indexer.safe_lengths().height,
            &prices.spot.cents.height,
            exit,
        )?;

        Ok(())
    }
}
