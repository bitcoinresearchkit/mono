use brk_error::Result;

use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;
use crate::Dependencies;

impl Vecs {
    fn compute_inner(
        &mut self,
        indexer: &Indexer,
        prices: &bitview_plugin_price::Vecs,
        mappings: &bitview_plugin_mappings::Vecs,
        blocks: &bitview_plugin_blocks::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        // Phase 1: Independent sub-modules in parallel
        let (r1, r2) = rayon::join(
            || super::ath::compute(&mut self.ath, indexer, prices, mappings, exit),
            || {
                rayon::join(
                    || super::range::compute(&mut self.range, indexer, prices, blocks, exit),
                    || {
                        super::moving_average::compute(
                            &mut self.moving_average,
                            indexer,
                            blocks,
                            prices,
                            exit,
                        )
                    },
                )
            },
        );
        r1?;
        r2.0?;
        r2.1?;

        // Phase 2: Stored volatility inputs derived from lazy 24h returns.
        super::returns::compute(&mut self.returns, indexer, blocks, exit)?;

        // Phase 3: Depends on moving_average
        super::technical::compute(
            &mut self.technical,
            indexer,
            prices,
            blocks,
            &self.moving_average,
            exit,
        )?;

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });
        Ok(())
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<Self::Output> {
        self.compute_inner(
            dependencies.indexer,
            dependencies.price,
            dependencies.mappings,
            dependencies.blocks,
            context.exit(),
        )
    }
}
