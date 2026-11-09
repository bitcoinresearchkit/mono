use bitview_plugin::{ComputePlugin, DependentContext, compute_items};
use brk_error::Result;

use super::Vecs;
use crate::Dependencies;

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: DependentContext<'_>,
    ) -> Result<()> {
        let starting_height = context.start().height;
        let Dependencies {
            mappings,
            difficulty_hashrate,
            coinbase_value,
            total_fees,
            price: prices,
        } = dependencies;
        let exit = context.exit();

        self.rewards.compute(
            &mappings.lookback,
            coinbase_value,
            total_fees,
            &prices.spot.cents.height,
            starting_height,
            exit,
        )?;

        self.hash.compute(
            &mappings.lookback,
            &difficulty_hashrate.height,
            &self.rewards.coinbase.sum._24h.sats.height,
            &self.rewards.coinbase.sum._24h.usd.height,
            starting_height,
            exit,
        )?;

        Ok(())
    }
}
