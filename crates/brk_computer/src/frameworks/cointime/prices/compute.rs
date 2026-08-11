use brk_error::Result;
use brk_indexer::Indexer;
use brk_types::Cents;
use vecdb::Exit;

use super::super::{activity, cap, supply};
use super::Vecs;
use crate::{distribution, price};

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        _prices: &price::Vecs,
        distribution: &distribution::Vecs,
        activity: &activity::Vecs,
        supply: &supply::Vecs,
        cap: &cap::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();
        let realized_price = &distribution.cohorts.realized.price.cohorts.all.cents.height;

        self.vaulted.cents.height.compute_transform2(
            starting_lengths.height,
            realized_price,
            &activity.vaultedness.height,
            |(i, price, vaultedness, ..)| {
                (i, Cents::from(f64::from(price) / f64::from(vaultedness)))
            },
            exit,
        )?;

        self.active.cents.height.compute_transform2(
            starting_lengths.height,
            realized_price,
            &activity.liveliness.height,
            |(i, price, liveliness, ..)| (i, Cents::from(f64::from(price) / f64::from(liveliness))),
            exit,
        )?;

        self.true_market_mean.cents.height.compute_transform2(
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
}
