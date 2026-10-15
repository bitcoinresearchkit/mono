use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use brk_error::Result;

use super::Vecs;
use crate::Dependencies;

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies {
            indexer,
            mappings,
            blocks,
            transactions,
            price: prices,
        } = dependencies;
        let exit = context.exit();

        // Block rewards (coinbase, subsidy, fee share, etc.)
        self.rewards
            .compute(indexer, &mappings.lookback, transactions, prices, exit)?;

        self.hashrate.compute(
            indexer,
            &mappings.lookback,
            &blocks.difficulty,
            &self.rewards.coinbase.sum._24h.sats.height,
            &self.rewards.coinbase.sum._24h.usd.height,
            exit,
        )?;

        Ok(())
    }
}
