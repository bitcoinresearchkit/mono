use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_types::Sats;
use vecdb::{Exit, VecIndex};

use super::Vecs;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    outputs: &bitview_plugin_outputs::Vecs,
    mining: &bitview_plugin_mining::Vecs,
    prices: &bitview_plugin_price::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, outputs, mining, prices, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        outputs: &bitview_plugin_outputs::Vecs,
        mining: &bitview_plugin_mining::Vecs,
        prices: &bitview_plugin_price::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        self.total.compute_from_pair(
            starting_height,
            &prices.spot.cents.height,
            &outputs.value.op_return.block.sats,
            &mining.rewards.unclaimed.block.sats,
            |height, op_return, unclaimed| {
                let genesis = if height.to_usize() == 0 {
                    Sats::FIFTY_BTC
                } else {
                    Sats::ZERO
                };
                genesis + op_return + unclaimed
            },
            exit,
        )?;
        Ok(())
    }
}
