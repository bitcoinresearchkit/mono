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
        mappings: &bitview_plugin_mappings::Vecs,
        blocks: &bitview_plugin_blocks::Vecs,
        transactions: &bitview_plugin_transactions::Vecs,
        prices: &bitview_plugin_price::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        // Block rewards (coinbase, subsidy, fee_dominance, etc.)
        super::rewards::compute(
            &mut self.rewards,
            indexer,
            mappings,
            &blocks.lookback,
            transactions,
            prices,
            exit,
        )?;

        super::hashrate::compute(
            &mut self.hashrate,
            indexer,
            &blocks.count,
            &blocks.lookback,
            &blocks.difficulty,
            &self.rewards.coinbase.sum._24h.sats.height,
            &self.rewards.coinbase.sum._24h.usd.height,
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
            dependencies.mappings,
            dependencies.blocks,
            dependencies.transactions,
            dependencies.price,
            context.exit(),
        )
    }
}
