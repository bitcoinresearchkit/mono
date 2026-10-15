use bitview_cohort::Age;
use bitview_primitives::{CoinDays, CostBasisSnapshot, Count, SupplyState};
use brk_types::{Bitcoin, Cents, CentsSats, CentsSigned, Sats};

use super::{
    SendPrecomputed, UnrealizedState,
    cost_basis::{CostBasisData, CostBasisOps, MinimalRealizedState, RealizedOps},
};
use crate::metrics::RealizedBlockData;

/// A UTXO cohort's accounting state.
pub type UTXOCohortState<R, C> = CohortState<R, C>;
/// A UTXO cohort's accounting state with its cost basis distribution.
pub type MappedUTXOCohortState<R> = CohortState<R, CostBasisData>;

#[derive(Default)]
pub struct CohortState<R: RealizedOps, C: CostBasisOps> {
    pub supply: SupplyState,
    pub realized: R,
    pub sent: Sats,
    spent_utxo_count: u64,
    satdays_destroyed: Sats,
    cost_basis: C,
}

impl<R: RealizedOps, C: CostBasisOps> CohortState<R, C> {
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
        let profit = self.realized.profit();
        let loss = self.realized.loss();
        RealizedBlockData {
            cap_raw: self.realized.cap_raw(),
            supply: self.supply.value,
            cap: self.realized.cap(),
            profit,
            loss,
            net_pnl: CentsSigned::new(profit.inner() as i64 - loss.inner() as i64),
            value_destroyed: self.realized.value_destroyed(),
        }
    }

    pub fn apply_pending(&mut self) {
        self.cost_basis.apply_pending();
    }

    pub fn reset_single_iteration_values(&mut self) {
        self.sent = Sats::ZERO;
        self.spent_utxo_count = 0;
        if R::TRACK_ACTIVITY {
            self.satdays_destroyed = Sats::ZERO;
        }
        self.realized.reset_single_iteration_values();
    }

    pub fn increment_snapshot(&mut self, s: &CostBasisSnapshot) {
        self.supply += &s.supply_state;

        if s.supply_state.value > Sats::ZERO {
            self.realized
                .increment_snapshot(s.price_sats, s.capitalized_cap_raw);
            self.cost_basis
                .increment(s.realized_price, s.supply_state.value);
        }
    }

    pub fn decrement_snapshot(&mut self, s: &CostBasisSnapshot) {
        self.supply -= &s.supply_state;

        if s.supply_state.value > Sats::ZERO {
            self.realized
                .decrement_snapshot(s.price_sats, s.capitalized_cap_raw);
            self.cost_basis
                .decrement(s.realized_price, s.supply_state.value);
        }
    }

    /// One received output. Several cohorts receiving the same output share one snapshot's
    /// products through [`Self::increment_snapshot`] instead.
    pub fn receive_utxo(&mut self, supply: &SupplyState, price: Cents) {
        self.supply += supply;

        if supply.value > Sats::ZERO {
            self.realized.increment(price, supply.value);
            self.cost_basis.increment(price, supply.value);
        }
    }

    pub fn send_utxo_precomputed(&mut self, supply: &SupplyState, pre: &SendPrecomputed) {
        self.supply -= supply;
        self.sent += pre.sats;
        self.spent_utxo_count += supply.utxo_count;
        if R::TRACK_ACTIVITY {
            self.satdays_destroyed += pre.age.satdays_destroyed(pre.sats);
        }

        self.realized.send(
            pre.sats,
            pre.current_ps,
            pre.prev_ps,
            pre.ath_ps,
            pre.prev_capitalized_cap,
        );

        self.cost_basis.decrement(pre.prev_price, pre.sats);
    }

    pub fn send_utxo(
        &mut self,
        supply: &SupplyState,
        current_price: Cents,
        prev_price: Cents,
        ath: Cents,
        age: Age,
    ) {
        if let Some(pre) = SendPrecomputed::new(supply, current_price, prev_price, ath, age) {
            self.send_utxo_precomputed(supply, &pre);
        } else if supply.utxo_count > 0 {
            self.supply -= supply;
            self.spent_utxo_count += supply.utxo_count;
        }
    }
}

impl<R: RealizedOps> CohortState<R, CostBasisData> {
    pub fn finish_restore(&mut self) {
        self.cost_basis.finish_restore();
    }

    pub(crate) fn compute_unrealized_state(&mut self, height_price: Cents) -> UnrealizedState {
        self.cost_basis.compute_unrealized_state(height_price)
    }
}

impl CohortState<MinimalRealizedState, ()> {
    pub fn send_minimal(
        &mut self,
        supply: &SupplyState,
        previous: CentsSats,
        profit: CentsSats,
        loss: CentsSats,
    ) {
        if supply.utxo_count == 0 {
            return;
        }
        self.supply -= supply;
        self.sent += supply.value;
        self.spent_utxo_count += supply.utxo_count;
        self.realized.apply_spends(previous, profit, loss);
    }

    pub fn increment_addr(&mut self, supply: &SupplyState, cap: CentsSats) {
        self.supply += supply;

        if supply.value.is_not_zero() {
            self.realized.increment_cap(cap);
        }
    }

    pub fn decrement_addr(&mut self, supply: &SupplyState, cap: CentsSats) {
        self.supply -= supply;

        if supply.value.is_not_zero() {
            self.realized.decrement_cap(cap);
        }
    }

    pub fn send_addr(&mut self, supply: &SupplyState, current_price: Cents, prev_ps: CentsSats) {
        if supply.utxo_count == 0 {
            return;
        }

        self.supply -= supply;

        if supply.value == Sats::ZERO {
            return;
        }

        self.sent += supply.value;

        let sats = supply.value;
        let current_ps = CentsSats::from_price_sats(current_price, sats);
        self.realized.realize_spend(current_ps, prev_ps);
    }
}
