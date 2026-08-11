use std::path::Path;

use brk_error::Result;
use brk_types::{Bitcoin, Cents, CentsSigned, Sats, StoredF64, StoredU64, SupplyState};
use derive_more::{Deref, DerefMut};

use super::super::CohortState;
use super::super::cost_basis::{CostBasisOps, RealizedOps};
use crate::distribution::metrics::RealizedBlockData;

#[derive(Deref, DerefMut)]
pub struct UTXOCohortState<R: RealizedOps, C: CostBasisOps>(pub CohortState<R, C>);

impl<R: RealizedOps, C: CostBasisOps> UTXOCohortState<R, C> {
    pub fn new(path: &Path, name: &str) -> Self {
        Self(CohortState::new(path, name))
    }

    pub fn reset_cost_basis_data_if_needed(&mut self) -> Result<()> {
        self.0.reset_cost_basis_data_if_needed()
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
    pub fn output_counts(&self) -> (StoredU64, StoredU64) {
        (
            StoredU64::from(self.supply.utxo_count),
            StoredU64::from(self.spent_utxo_count),
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
        let price = cap_raw
            .as_u128()
            .checked_div(supply.as_u128())
            .map(|price| Cents::new(price as u64))
            .unwrap_or_default();

        RealizedBlockData {
            cap_raw,
            supply,
            cap,
            price,
            profit,
            loss,
            net_pnl: CentsSigned::new(profit.inner() as i64 - loss.inner() as i64),
            value_destroyed: self.realized.value_destroyed(),
        }
    }
}
