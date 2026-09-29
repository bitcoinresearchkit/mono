use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;

use super::{Vecs, hashrate, rewards};
use crate::Dependencies;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn compute(
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

        self.db.sync_bg_tasks()?;

        // Block rewards (coinbase, subsidy, fee_dominance, etc.)
        rewards::compute(
            &mut self.rewards,
            indexer,
            mappings,
            &blocks.lookback,
            transactions,
            prices,
            exit,
        )?;

        hashrate::compute(
            &mut self.hashrate,
            indexer,
            &blocks.count,
            &blocks.lookback,
            &blocks.difficulty,
            &self.rewards.coinbase.rolling.sum._24h.sats.height,
            &self.rewards.coinbase.rolling.sum._24h.usd.height,
            exit,
        )?;

        context.compact_database(&self.db);
        Ok(())
    }
}
