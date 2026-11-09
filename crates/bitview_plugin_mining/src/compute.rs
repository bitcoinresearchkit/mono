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
        let starting_height = context.start().height;
        let Dependencies {
            mappings,
            blocks,
            transactions,
            price: prices,
        } = dependencies;
        let exit = context.exit();

        self.rewards.compute(
            &mappings.lookback,
            transactions,
            prices,
            starting_height,
            exit,
        )?;

        self.hash.compute(
            &mappings.lookback,
            &blocks.difficulty,
            &self.rewards.coinbase.sum._24h.sats.height,
            &self.rewards.coinbase.sum._24h.usd.height,
            starting_height,
            exit,
        )?;

        Ok(())
    }
}
