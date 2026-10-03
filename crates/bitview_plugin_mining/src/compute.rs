use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
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
            blocks,
            transactions,
            price: prices,
        } = dependencies;
        let exit = context.exit();

        // Block rewards (coinbase, subsidy, fee_dominance, etc.)
        self.rewards
            .compute(indexer, &blocks.lookback, transactions, prices, exit)?;

        self.hashrate.compute(
            indexer,
            &blocks.count,
            &blocks.lookback,
            &blocks.difficulty,
            &self.rewards.coinbase.rolling.sum._24h.sats.height,
            &self.rewards.coinbase.rolling.sum._24h.usd.height,
            exit,
        )?;

        Ok(())
    }
}
