use bitview_plugin_distribution_age::Vecs as AgeVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Bitcoin, Dollars, StoredF64};
use vecdb::ReadableVec;

use super::{super::activity, Vecs};

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    prices: &PriceVecs,
    distribution_age: &AgeVecs,
    activity: &activity::Vecs,
    exit: &Exit,
) -> Result<()> {
    let starting_height = indexer.safe_lengths().height;
    let coinblocks_destroyed = &distribution_age.coinblocks_destroyed;
    let coindays_destroyed = &distribution_age
        .cohorts
        .activity
        .coindays_destroyed
        .cohorts
        .all;
    let circulating_supply = &distribution_age.cohorts.supply.total.cohorts.all.btc.height;

    for (target, source) in [
        (
            &mut vecs.destroyed.cumulative.height,
            &coinblocks_destroyed.block,
        ),
        (
            &mut vecs.created.cumulative.height,
            &activity.coinblocks_created.block,
        ),
        (
            &mut vecs.stored.cumulative.height,
            &activity.coinblocks_stored.block,
        ),
    ] {
        target.compute_cumulative_transformed_binary(
            starting_height,
            &prices.spot.usd.height,
            source,
            |price, value| StoredF64::from(f64::from(price) * f64::from(value)),
            exit,
        )?;
    }

    // VOCDD: Value of Coin Days Destroyed = price × (CDD / circulating_supply)
    // Supply-adjusted to account for growing supply over time
    // This is a key input for Reserve Risk / HODL Bank calculation
    let mut cumulative = None;
    vecs.vocdd.cumulative.height.compute_transform3(
        starting_height,
        &prices.spot.usd.height,
        &coindays_destroyed.block,
        circulating_supply,
        |(i, price, cdd, supply, this): (_, Dollars, StoredF64, Bitcoin, _)| {
            let cumulative = cumulative.get_or_insert_with(|| {
                i.decremented()
                    .and_then(|height| this.collect_one(height))
                    .unwrap_or_default()
            });
            let supply_f64 = f64::from(supply);
            let value = if supply_f64 == 0.0 {
                StoredF64::from(0.0)
            } else {
                // VOCDD = price × (CDD / supply)
                StoredF64::from(f64::from(price) * f64::from(cdd) / supply_f64)
            };
            *cumulative += value;
            (i, *cumulative)
        },
        exit,
    )?;

    Ok(())
}
