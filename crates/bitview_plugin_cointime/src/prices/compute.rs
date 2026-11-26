use bitview_plugin_holders::Vecs as HoldersVecs;
use brk_error::Result;
use brk_exit::Exit;
use bitview_primitives::{CoinDays, Float64};
use bitview_vecs::first_price_height;
use brk_types::{Bitcoin, Cents, Dollars, Height, Timestamp};
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

    /// The running sums the transferred, balanced, terminal, CVDD and delta prices divide.
    pub(crate) fn compute_sums(
        &mut self,
        from: Height,
        spot: &impl ReadableVec<Height, Dollars>,
        coindays_destroyed: &impl ReadableVec<Height, CoinDays>,
        market_cap: &impl ReadableVec<Height, Cents>,
        timestamps: &impl ReadableVec<Height, Timestamp>,
        exit: &Exit,
    ) -> Result<()> {
        self.value_days_destroyed_sum.compute_cumulative_transformed_binary(
            from,
            spot,
            coindays_destroyed,
            |price, cdd| Float64::from(f64::from(price) * f64::from(cdd)),
            exit,
        )?;
        // The mean over time since the first price, each block's cap held until the next: a running
        // mean, its last value, cap and time picked up where the last update stopped (none before
        // the first price, or on a chain without one).
        let first = first_price_height().map(Height::from);
        let mut state = None;
        self.average_cap.compute_transform2(
            from,
            market_cap,
            timestamps,
            |(height, cap, at, this): (Height, Cents, Timestamp, _)| {
                let Some(first) = first.filter(|first| height >= *first) else {
                    return Dollars::NAN;
                };
                let usd = |cap: Cents| f64::from(Dollars::from(cap));
                let cap = usd(cap);
                let (mean, last_cap, last_at, start) = state.get_or_insert_with(|| {
                    let start = timestamps.collect_one(first).expect("timestamps cover the range");
                    match height.decremented().filter(|before| *before >= first) {
                        Some(before) => (
                            f64::from(this.collect_one(before).expect("the mean covers the range")),
                            usd(market_cap.collect_one(before).expect("caps cover the range")),
                            timestamps.collect_one(before).expect("timestamps cover the range"),
                            start,
                        ),
                        None => (cap, cap, at, start),
                    }
                });
                let elapsed = at.days_since(*start);
                if elapsed > 0.0 {
                    *mean += (*last_cap - *mean) * at.days_since(*last_at) / elapsed;
                }
                (*last_cap, *last_at) = (cap, at);
                Dollars::from(*mean)
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
