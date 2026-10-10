use bitview_plugin_holders::Vecs as HoldersVecs;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height};
use vecdb::ReadableVec;

use super::{
    super::{activity, cap, supply},
    Vecs,
};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        from: Height,
        holders: &HoldersVecs,
        activity: &activity::Vecs,
        supply: &supply::Vecs,
        cap: &cap::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let realized_price = &holders.cohorts.all.cost_basis.per_coin.avg.cents.height;

        self.vaulted_cents.compute_transform2(
            from,
            realized_price,
            &activity.vaultedness.height,
            |(i, price, vaultedness, ..)| {
                (i, Cents::from(f64::from(price) / f64::from(vaultedness)))
            },
            exit,
        )?;

        self.active_cents.compute_transform2(
            from,
            realized_price,
            &activity.liveliness.height,
            |(i, price, liveliness, ..)| (i, Cents::from(f64::from(price) / f64::from(liveliness))),
            exit,
        )?;

        self.true_market_mean_cents.compute_transform2(
            from,
            &cap.investor.cents.height,
            &supply.active.btc.height,
            |(i, cap_cents, supply_btc, ..)| {
                (i, Cents::from(f64::from(cap_cents) / f64::from(supply_btc)))
            },
            exit,
        )?;

        Ok(())
    }

    /// Spot divided by each price, stored.
    pub(crate) fn compute_ratios(
        &mut self,
        from: Height,
        spot: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.vaulted.compute_ratio(from, spot, exit)?;
        self.active.compute_ratio(from, spot, exit)?;
        self.true_market_mean.compute_ratio(from, spot, exit)?;
        self.cointime.compute_ratio(from, spot, exit)
    }
}
