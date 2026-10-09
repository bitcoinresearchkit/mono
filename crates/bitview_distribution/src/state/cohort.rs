use bitview_cohort::Age;
use bitview_primitives::{CostBasisSnapshot, SupplyState};
use brk_types::{Cents, CentsSats, Sats};

use super::{
    SendPrecomputed,
    cost_basis::{CostBasisOps, MinimalRealizedState, RealizedOps},
};

pub struct CohortState<R: RealizedOps, C: CostBasisOps> {
    pub supply: SupplyState,
    pub realized: R,
    pub sent: Sats,
    pub(crate) spent_utxo_count: u64,
    pub(crate) satdays_destroyed: Sats,
    pub(crate) cost_basis: C,
}

impl<R: RealizedOps, C: CostBasisOps> CohortState<R, C> {
    pub fn new(cost_basis: C) -> Self {
        Self {
            supply: SupplyState::default(),
            realized: R::default(),
            sent: Sats::ZERO,
            spent_utxo_count: 0,
            satdays_destroyed: Sats::ZERO,
            cost_basis,
        }
    }

    pub fn init_cost_basis(&mut self) {
        self.cost_basis.init();
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

    pub fn receive_utxo(&mut self, supply: &SupplyState, price: Cents) {
        self.receive_utxo_snapshot(supply, &CostBasisSnapshot::from_utxo(price, supply));
    }

    /// Like receive_utxo but takes a pre-computed snapshot to avoid redundant multiplication
    /// when the same supply/price is used across multiple cohorts.
    pub fn receive_utxo_snapshot(&mut self, supply: &SupplyState, snapshot: &CostBasisSnapshot) {
        self.supply += supply;

        if supply.value > Sats::ZERO {
            self.realized.receive(snapshot.realized_price, supply.value);

            self.cost_basis
                .increment(snapshot.realized_price, supply.value);
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
