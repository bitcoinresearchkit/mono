use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::Float64;
use brk_error::Result;
use brk_exit::Exit;

use super::{super::activity, Vecs};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        prices: &PriceVecs,
        age: &AgeVecs,
        activity: &activity::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let coinblocks_destroyed = &age.coinblocks_destroyed;

        for (target, source) in [
            (
                &mut self.destroyed.cumulative.height,
                &coinblocks_destroyed.block,
            ),
            (
                &mut self.created.cumulative.height,
                &activity.coinblocks_created.block,
            ),
            (
                &mut self.stored.cumulative.height,
                &activity.coinblocks_stored.block,
            ),
        ] {
            target.compute_cumulative_transformed_binary(
                starting_height,
                &prices.spot.usd.height,
                source,
                |price, value| Float64::from(f64::from(price) * f64::from(value)),
                exit,
            )?;
        }

        Ok(())
    }
}
