use brk_error::Result;

use bitview_plugin::ComputePlugin;
use bitview_plugin_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;

impl Vecs {
    fn compute_inner(
        &mut self,
        indexer: &Indexer,
        inputs: &bitview_plugin_inputs::Vecs,
        indexes: &bitview_plugin_indexes::Vecs,
        blocks: &bitview_plugin_blocks::Vecs,
        prices: &bitview_plugin_price::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        let ((r1, r2), (r3, r4)) = rayon::join(
            || {
                rayon::join(
                    || super::count::compute(&mut self.count, indexer, &blocks.lookback, exit),
                    || super::features::compute(&mut self.features, indexer, exit),
                )
            },
            || {
                rayon::join(
                    || super::versions::compute(&mut self.versions, indexer, exit),
                    || super::size::compute(&mut self.size, indexer, indexes, exit),
                )
            },
        );
        r1?;
        r2?;
        r3?;
        r4?;

        super::sigops::compute(&mut self.sigops, indexer, indexes, exit)?;

        super::fees::compute(
            &mut self.fees,
            indexer,
            &inputs.value,
            indexes,
            &self.size,
            exit,
        )?;

        super::patterns::compute(&mut self.patterns, indexer, &inputs.value, indexes, exit)?;

        super::policy::compute(&mut self.policy, indexer, indexes, &self.fees, exit)?;

        super::volume::compute(&mut self.volume, indexer, indexes, prices, &self.fees, exit)?;

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });
        Ok(())
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = crate::Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        exit: &Exit,
    ) -> Result<Self::Output> {
        self.compute_inner(
            dependencies.indexer,
            dependencies.inputs,
            dependencies.indexes,
            dependencies.blocks,
            dependencies.price,
            exit,
        )
    }
}
