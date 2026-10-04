use bitview_primitives::{Count, StoredF64, SupplyState};
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

    /// Reset state for fresh start.
    pub fn reset(&mut self) {
        self.0.supply = SupplyState::default();
        self.0.sent = Sats::ZERO;
        self.0.spent_utxo_count = 0;
        self.0.satdays_destroyed = Sats::ZERO;
        self.0.realized = R::default();
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
    pub fn core_activity(&self) -> (StoredF64, Sats, Sats) {
        (
            StoredF64::from(Bitcoin::from(self.satdays_destroyed)),
            self.realized.sent_in_profit(),
            self.realized.sent_in_loss(),
        )
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
