use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::{CoinBlocks, DollarBlocks};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Dollars, Height};
use vecdb::{ComputeVec, ReadableVec};

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
        let starting_height = indexer.compute_start().height;
        let price = &prices.spot.usd.height;
        compute_priced(
            &mut self.destroyed.cumulative.height,
            starting_height,
            price,
            &age.coinblocks_destroyed.block,
            exit,
        )?;
        compute_priced(
            &mut self.created.cumulative.height,
            starting_height,
            price,
            &activity.coinblocks_created.block,
            exit,
        )?;
        compute_priced(
            &mut self.stored.cumulative.height,
            starting_height,
            price,
            &activity.coinblocks_stored.block,
            exit,
        )
    }
}

fn compute_priced(
    target: &mut impl ComputeVec<I = Height, T = DollarBlocks>,
    from: Height,
    price: &impl ReadableVec<Height, Dollars>,
    source: &impl ReadableVec<Height, CoinBlocks>,
    exit: &Exit,
) -> Result<()> {
    Ok(target.compute_cumulative_transformed_binary(
        from,
        price,
        source,
        |price, value| DollarBlocks::from(f64::from(price) * f64::from(value)),
        exit,
    )?)
}
