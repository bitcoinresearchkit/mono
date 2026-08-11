use std::{collections::BTreeMap, path::Path};

use brk_error::Result;
use brk_types::{
    Age, Cents, CentsCompact, CentsSats, CostBasisSnapshot, Height, Sats, SupplyState,
};

use super::{
    SendPrecomputed,
    cost_basis::{
        Accumulate, CostBasisData, CostBasisOps, CostBasisRaw, MinimalRealizedState, RealizedOps,
        UnrealizedState,
    },
    pending::PendingDelta,
};

pub struct CohortState<R: RealizedOps, C: CostBasisOps> {
    pub supply: SupplyState,
    pub realized: R,
    pub sent: Sats,
    pub spent_utxo_count: u64,
    pub satdays_destroyed: Sats,
    cost_basis: C,
}

impl<R: RealizedOps, C: CostBasisOps> CohortState<R, C> {
    pub fn new(path: &Path, name: &str) -> Self {
        Self {
            supply: SupplyState::default(),
            realized: R::default(),
            sent: Sats::ZERO,
            spent_utxo_count: 0,
            satdays_destroyed: Sats::ZERO,
            cost_basis: C::create(path, name),
        }
    }

    pub fn import_at_or_before(&mut self, height: Height) -> Result<Height> {
        self.cost_basis.import_at_or_before(height)
    }

    /// Restore realized cap from cost_basis after import.
    pub fn restore_realized_cap(&mut self) {
        self.realized.set_cap_raw(self.cost_basis.cap_raw());
        self.realized
            .set_capitalized_cap_raw(self.cost_basis.capitalized_cap_raw());
    }

    pub fn reset_cost_basis_data_if_needed(&mut self) -> Result<()> {
        self.cost_basis.clean()?;
        self.cost_basis.init();
        Ok(())
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
            self.cost_basis.increment(
                s.realized_price,
                s.supply_state.value,
                s.price_sats,
                s.capitalized_cap_raw,
            );
        }
    }

    pub fn decrement_snapshot(&mut self, s: &CostBasisSnapshot) {
        self.supply -= &s.supply_state;

        if s.supply_state.value > Sats::ZERO {
            self.realized
                .decrement_snapshot(s.price_sats, s.capitalized_cap_raw);
            self.cost_basis.decrement(
                s.realized_price,
                s.supply_state.value,
                s.price_sats,
                s.capitalized_cap_raw,
            );
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

            self.cost_basis.increment(
                snapshot.realized_price,
                supply.value,
                snapshot.price_sats,
                snapshot.capitalized_cap_raw,
            );
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

        self.cost_basis.decrement(
            pre.prev_price,
            pre.sats,
            pre.prev_ps,
            pre.prev_capitalized_cap,
        );
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

    pub fn write(&mut self, height: Height, cleanup: bool) -> Result<()> {
        self.cost_basis.write(height, cleanup)
    }
}

impl CohortState<MinimalRealizedState, CostBasisRaw> {
    pub fn increment_addr(&mut self, supply: &SupplyState, cap: CentsSats) {
        self.supply += supply;

        if supply.value.is_not_zero() {
            self.realized.increment_cap(cap);
            self.cost_basis.increment_cap(cap);
        }
    }

    pub fn decrement_addr(&mut self, supply: &SupplyState, cap: CentsSats) {
        self.supply -= supply;

        if supply.value.is_not_zero() {
            self.realized.decrement_cap(cap);
            self.cost_basis.decrement_cap(cap);
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
        self.cost_basis.decrement_cap(prev_ps);
    }
}

/// Methods only available with CostBasisData (map + unrealized).
impl<R: RealizedOps, S: Accumulate> CohortState<R, CostBasisData<S>> {
    pub fn compute_unrealized_state(&mut self, height_price: Cents) -> UnrealizedState {
        self.cost_basis.compute_unrealized_state(height_price)
    }

    pub fn for_each_cost_basis_pending(&self, f: impl FnMut(&CentsCompact, &PendingDelta)) {
        self.cost_basis.for_each_pending(f);
    }

    pub fn cost_basis_map(&self) -> &BTreeMap<CentsCompact, Sats> {
        self.cost_basis.map()
    }
}
