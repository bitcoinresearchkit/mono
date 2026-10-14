use bitview_primitives::{CoinDays, Count};
use brk_types::{Bitcoin, CentsSigned, Sats};
use derive_more::{Deref, DerefMut};

use super::{
    super::CohortState,
    super::cost_basis::{CostBasisOps, RealizedOps},
};
use crate::metrics::RealizedBlockData;

#[derive(Deref, DerefMut)]
pub struct UTXOCohortState<R: RealizedOps, C: CostBasisOps>(pub CohortState<R, C>);

impl<R: RealizedOps, C: CostBasisOps> UTXOCohortState<R, C> {
    pub fn new(cost_basis: C) -> Self {
        Self(CohortState::new(cost_basis))
    }

    #[inline(always)]
    pub fn supply_value(&self) -> Sats {
        self.supply.value
    }

    #[inline(always)]
    pub fn output_counts(&self) -> (Count, Count) {
        (
            Count::from(self.supply.utxo_count),
            Count::from(self.spent_utxo_count),
        )
    }

    #[inline(always)]
    pub fn transfer_volume(&self) -> Sats {
        self.sent
    }

    #[inline(always)]
    pub(crate) fn coindays_destroyed(&self) -> CoinDays {
        CoinDays::new(f64::from(Bitcoin::from(self.satdays_destroyed)))
    }

    #[inline(always)]
    pub fn realized_block_data(&self) -> RealizedBlockData {
        let cap_raw = self.realized.cap_raw();
        let supply = self.supply.value;
        let cap = self.realized.cap();
        let profit = self.realized.profit();
        let loss = self.realized.loss();

        RealizedBlockData {
            cap_raw,
            supply,
            cap,
            profit,
            loss,
            net_pnl: CentsSigned::new(profit.inner() as i64 - loss.inner() as i64),
            value_destroyed: self.realized.value_destroyed(),
        }
    }
}
