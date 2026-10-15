use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_plugin_indexer::overwritten_output;
use brk_error::Result;
use brk_types::Sats;
use vecdb::VecIndex;

use crate::{Dependencies, Vecs};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies {
            indexer,
            outputs,
            mining,
            price,
        } = dependencies;

        self.burned.compute_from_pair(
            indexer.compute_start().height,
            &price.spot.cents.height,
            &outputs.value.op_return_value.block.sats,
            &mining.rewards.unclaimed.block.sats,
            |height, op_return, unclaimed| {
                let genesis = if height.to_usize() == 0 {
                    Sats::FIFTY_BTC
                } else {
                    Sats::ZERO
                };
                // A BIP30 duplicate coinbase made the output it overwrote unspendable.
                let overwritten =
                    overwritten_output(height).map_or(Sats::ZERO, |(_, lost)| lost.value);
                genesis + op_return + unclaimed + overwritten
            },
            context.exit(),
        )
    }
}
