use bitview_plugin_holders::Vecs as HoldersVecs;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Bitcoin, Cents, Height};
use vecdb::{ComputeVec, ReadableVec};

use super::{
    super::{cap, supply},
    Vecs,
};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        from: Height,
        holders: &HoldersVecs,
        supply: &supply::Vecs,
        cap: &cap::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        // Realized cap over each part of the supply, rounded once (the realized price over
        // liveliness divided an already rounded cent: ±45% in 2010-11).
        let realized_cap = &holders.cohorts.all.capital.total.cents.height;
        self.vaulted_cents.compute_transform2(
            from,
            realized_cap,
            &supply.vaulted.btc.height,
            |(_, cap, supply, ..)| per_coin(cap, supply),
            exit,
        )?;
        self.active_cents.compute_transform2(
            from,
            realized_cap,
            &supply.active.btc.height,
            |(_, cap, supply, ..)| per_coin(cap, supply),
            exit,
        )?;

        self.true_market_mean_cents.compute_transform2(
            from,
            &cap.investor.cents.height,
            &supply.active.btc.height,
            |(_, cap_cents, supply_btc, ..)| {
                let (cap, supply) = (f64::from(cap_cents), f64::from(supply_btc));
                if cap > 0.0 && supply > 0.0 {
                    Cents::from(cap / supply)
                } else {
                    Cents::NAN
                }
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

/// A cap's price per coin of a supply, undefined without supply (as the realized price): zero
/// before the first price, where every coin is valued at zero.
fn per_coin(cap: Cents, supply: Bitcoin) -> Cents {
    let supply = f64::from(supply);
    if supply > 0.0 {
        Cents::from(f64::from(cap) / supply)
    } else {
        Cents::NAN
    }
}
