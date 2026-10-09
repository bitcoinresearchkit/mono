use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::{CoinDays, Float64};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Bitcoin, Dollars};
use vecdb::ReadableVec;

use super::{super::activity, Vecs};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        prices: &PriceVecs,
        age: &AgeVecs,
        holders: &HoldersVecs,
        activity: &activity::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let coinblocks_destroyed = &age.coinblocks_destroyed;
        let coindays_destroyed = &holders.cohorts.all.activity.coindays_destroyed;
        let circulating_supply = &holders.cohorts.all.supply.total.btc.height;

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

        // VOCDD: Value of Coin Days Destroyed = price × (CDD / circulating_supply)
        // Supply-adjusted to account for growing supply over time
        // This is a key input for Reserve Risk / HODL Bank calculation
        let mut cumulative = None;
        self.vocdd.cumulative.height.compute_transform3(
            starting_height,
            &prices.spot.usd.height,
            &coindays_destroyed.block,
            circulating_supply,
            |(i, price, cdd, supply, this): (_, Dollars, CoinDays, Bitcoin, _)| {
                let cumulative = cumulative.get_or_insert_with(|| {
                    i.decremented()
                        .and_then(|height| this.collect_one(height))
                        .unwrap_or_default()
                });
                let supply_f64 = f64::from(supply);
                let value = if supply_f64 == 0.0 {
                    Float64::ZERO
                } else {
                    // VOCDD = price × (CDD / supply)
                    Float64::from(f64::from(price) * f64::from(cdd) / supply_f64)
                };
                *cumulative += value;
                (i, *cumulative)
            },
            exit,
        )?;

        Ok(())
    }
}
