use bitview_plugin_distribution_age::Vecs as AgeVecs;
use bitview_plugin_indexer::Indexer;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Cents;

use super::{
    super::{activity, cap, supply},
    Vecs,
};

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    distribution_age: &AgeVecs,
    activity: &activity::Vecs,
    supply: &supply::Vecs,
    cap: &cap::Vecs,
    exit: &Exit,
) -> Result<()> {
    let starting_lengths = indexer.safe_lengths();
    let realized_price = &distribution_age
        .cohorts
        .realized
        .price
        .cohorts
        .all
        .cents
        .height;

    vecs.vaulted.cents.height.compute_transform2(
        starting_lengths.height,
        realized_price,
        &activity.vaultedness.height,
        |(i, price, vaultedness, ..)| (i, Cents::from(f64::from(price) / f64::from(vaultedness))),
        exit,
    )?;

    vecs.active.cents.height.compute_transform2(
        starting_lengths.height,
        realized_price,
        &activity.liveliness.height,
        |(i, price, liveliness, ..)| (i, Cents::from(f64::from(price) / f64::from(liveliness))),
        exit,
    )?;

    vecs.true_market_mean.cents.height.compute_transform2(
        starting_lengths.height,
        &cap.investor.cents.height,
        &supply.active.btc.height,
        |(i, cap_cents, supply_btc, ..)| {
            (i, Cents::from(f64::from(cap_cents) / f64::from(supply_btc)))
        },
        exit,
    )?;

    Ok(())
}
