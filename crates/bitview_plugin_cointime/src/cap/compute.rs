use bitview_plugin_distribution_age::Vecs as AgeVecs;
use bitview_plugin_indexer::Indexer;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Dollars;

use super::{
    super::{activity, value},
    Vecs,
};

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    distribution_age: &AgeVecs,
    activity: &activity::Vecs,
    value: &value::Vecs,
    exit: &Exit,
) -> Result<()> {
    let starting_lengths = indexer.safe_lengths();
    let realized_cap_cents = &distribution_age
        .cohorts
        .realized
        .cap
        .cohorts
        .all
        .cents
        .height;
    let circulating_supply = &distribution_age.cohorts.supply.total.cohorts.all.btc.height;

    vecs.investor.cents.height.compute_subtract(
        starting_lengths.height,
        realized_cap_cents,
        &vecs.thermo.cents.height,
        exit,
    )?;

    vecs.vaulted.cents.height.compute_multiply(
        starting_lengths.height,
        realized_cap_cents,
        &activity.vaultedness.height,
        exit,
    )?;

    vecs.active.cents.height.compute_multiply(
        starting_lengths.height,
        realized_cap_cents,
        &activity.liveliness.height,
        exit,
    )?;

    // cointime_cap = (cointime_value_destroyed_cumulative * circulating_supply) / coinblocks_stored_cumulative
    vecs.cointime.cents.height.compute_transform3(
        starting_lengths.height,
        &value.destroyed.cumulative.height,
        circulating_supply,
        &activity.coinblocks_stored.cumulative.height,
        |(i, destroyed, supply, stored, ..)| {
            let destroyed: f64 = *destroyed;
            let supply: f64 = supply.into();
            let stored: f64 = *stored;
            let usd = Dollars::from(destroyed * supply / stored);
            (i, usd.to_cents())
        },
        exit,
    )?;

    // AVIV = active_cap / investor_cap
    vecs.aviv.compute_ratio(
        &starting_lengths,
        &vecs.active.cents.height,
        &vecs.investor.cents.height,
        exit,
    )?;

    Ok(())
}
