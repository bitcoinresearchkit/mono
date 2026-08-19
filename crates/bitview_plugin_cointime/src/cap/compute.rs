use brk_error::Result;

use brk_indexer::Indexer;
use brk_types::Dollars;
use vecdb::Exit;

use super::super::{activity, value};
use super::Vecs;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    distribution: &bitview_plugin_distribution::Vecs,
    activity: &activity::Vecs,
    value: &value::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, distribution, activity, value, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        distribution: &bitview_plugin_distribution::Vecs,
        activity: &activity::Vecs,
        value: &value::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();
        let realized_cap_cents = &distribution.cohorts.realized.cap.cohorts.all.cents.height;
        let circulating_supply = &distribution.cohorts.supply.total.cohorts.all.btc.height;

        self.investor.cents.height.compute_subtract(
            starting_lengths.height,
            realized_cap_cents,
            &self.thermo.cents.height,
            exit,
        )?;

        self.vaulted.cents.height.compute_multiply(
            starting_lengths.height,
            realized_cap_cents,
            &activity.vaultedness.height,
            exit,
        )?;

        self.active.cents.height.compute_multiply(
            starting_lengths.height,
            realized_cap_cents,
            &activity.liveliness.height,
            exit,
        )?;

        // cointime_cap = (cointime_value_destroyed_cumulative * circulating_supply) / coinblocks_stored_cumulative
        self.cointime.cents.height.compute_transform3(
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
        self.aviv.compute_ratio(
            &starting_lengths,
            &self.active.cents.height,
            &self.investor.cents.height,
            exit,
        )?;

        Ok(())
    }
}
