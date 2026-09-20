use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use rayon::join;

use super::{
    Vecs, count, features, fees, inscription, patterns, policy, sigops, size, versions, volume,
};
use crate::Dependencies;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<Self::Output> {
        let Dependencies {
            indexer,
            inputs,
            mappings,
            blocks,
            price: prices,
        } = dependencies;
        let exit = context.exit();
        self.db.sync_bg_tasks()?;

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
            exit,
        )?;

        inscription::compute(&mut self.inscription, indexer, mappings, &self.fees, exit)?;

        patterns::compute(&mut self.patterns, indexer, &inputs.value, mappings, exit)?;

        policy::compute(&mut self.policy, indexer, mappings, &self.fees, exit)?;

        volume::compute(
            &mut self.volume,
            indexer,
            mappings,
            prices,
            &self.fees,
            exit,
        )?;

        context.compact_database(&self.db);
        Ok(())
    }
}
