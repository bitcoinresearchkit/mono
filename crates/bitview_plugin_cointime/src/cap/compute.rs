use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_primitives::Lengths;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{CentsSigned, Dollars};
use vecdb::ComputeVec;

use super::{
    super::{activity, value},
    Vecs,
};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        holders: &HoldersVecs,
        activity: &activity::Vecs,
        value: &value::Vecs,
        starting_lengths: Lengths,
        exit: &Exit,
    ) -> Result<()> {
        let realized_cap_cents = &holders.cohorts.all.capital.total.cents.height;
        let circulating_supply = &holders.cohorts.all.supply.total.btc.height;

        self.investor.cents.height.compute_transform2(
            starting_lengths.height,
            realized_cap_cents,
            &self.thermo.cents.height,
            |(_, realized, thermo, ..)| {
                CentsSigned::new(realized.inner() as i64 - thermo.inner() as i64)
            },
            exit,
        )?;

        self.cointime.cents.height.compute_transform3(
            starting_lengths.height,
            &value.destroyed.cumulative.height,
            circulating_supply,
            &activity.coinblocks_stored.cumulative.height,
            |(_, destroyed, supply, stored, ..)| {
                let destroyed: f64 = *destroyed;
                let supply: f64 = supply.into();
                let stored: f64 = *stored;
                Dollars::from(destroyed * supply / stored).to_cents()
            },
            exit,
        )?;

        Ok(())
    }
}
