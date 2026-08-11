use std::path::Path;

use brk_error::Result;
use brk_types::{Cents, FundedAddrData, Sats, SupplyState};
use vecdb::unlikely;

use super::super::CohortState;
use super::super::cost_basis::{CostBasisRaw, MinimalRealizedState};

/// Mutable state for one address balance cohort.
pub struct AddrCohortState {
    pub addr_count: u64,
    pub inner: CohortState<MinimalRealizedState, CostBasisRaw>,
}

impl AddrCohortState {
    pub fn new(path: &Path, name: &str) -> Self {
        Self {
            addr_count: 0,
            inner: CohortState::new(path, name),
        }
    }

    /// Reset state for fresh start.
    pub fn reset(&mut self) {
        self.addr_count = 0;
        self.inner.supply = SupplyState::default();
        self.inner.sent = Sats::ZERO;
        self.inner.spent_utxo_count = 0;
        self.inner.satdays_destroyed = Sats::ZERO;
        self.inner.realized = MinimalRealizedState::default();
    }

    pub fn send(
        &mut self,
        addr_data: &mut FundedAddrData,
        value: Sats,
        current_price: Cents,
        prev_price: Cents,
    ) -> Result<()> {
        let prev_ps = addr_data.send(value, prev_price)?;

        self.inner.send_addr(
            &SupplyState {
                utxo_count: 1,
                value,
            },
            current_price,
            prev_ps,
        );

        Ok(())
    }

    pub fn receive_outputs(
        &mut self,
        addr_data: &mut FundedAddrData,
        value: Sats,
        price: Cents,
        output_count: u32,
    ) {
        let cap = addr_data.receive_outputs(value, price, output_count);

        self.inner.increment_addr(
            &SupplyState {
                utxo_count: output_count as u64,
                value,
            },
            cap,
        );
    }

    pub fn add(&mut self, addr_data: &FundedAddrData) {
        self.addr_count += 1;
        let supply = SupplyState::from(addr_data);
        self.inner
            .increment_addr(&supply, addr_data.realized_cap_raw);
    }

    pub fn subtract(&mut self, addr_data: &FundedAddrData) {
        let supply = SupplyState::from(addr_data);

        // Check for potential underflow before it happens
        if unlikely(self.inner.supply.utxo_count < supply.utxo_count) {
            panic!(
                "AddrCohortState::subtract underflow!\n\
                Cohort state: addr_count={}, supply={}\n\
                Addr being subtracted: {}\n\
                Addr supply: {}\n\
                Realized price: {}\n\
                This means the addr is not properly tracked in this cohort.",
                self.addr_count,
                self.inner.supply,
                addr_data,
                supply,
                addr_data.realized_price()
            );
        }
        if unlikely(self.inner.supply.value < supply.value) {
            panic!(
                "AddrCohortState::subtract value underflow!\n\
                Cohort state: addr_count={}, supply={}\n\
                Addr being subtracted: {}\n\
                Addr supply: {}\n\
                Realized price: {}\n\
                This means the addr is not properly tracked in this cohort.",
                self.addr_count,
                self.inner.supply,
                addr_data,
                supply,
                addr_data.realized_price()
            );
        }

        self.addr_count = self.addr_count.checked_sub(1).unwrap_or_else(|| {
            panic!(
                "AddrCohortState::subtract addr_count underflow! addr_count=0\n\
                Addr being subtracted: {}\n\
                Realized price: {}",
                addr_data,
                addr_data.realized_price()
            )
        });

        self.inner
            .decrement_addr(&supply, addr_data.realized_cap_raw);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distribution::state::cost_basis::RealizedOps;

    #[test]
    fn address_updates_preserve_supply_cap_and_transfer_state() {
        let mut cohort = AddrCohortState::new(Path::new(""), "test");
        let mut addr = FundedAddrData::default();

        addr.receive(Sats::ONE_BTC, Cents::new(10_000));
        cohort.add(&addr);
        cohort.receive_outputs(&mut addr, Sats::new(50_000_000), Cents::new(20_000), 1);
        cohort
            .send(
                &mut addr,
                Sats::new(25_000_000),
                Cents::new(15_000),
                Cents::new(10_000),
            )
            .unwrap();

        assert_eq!(
            cohort.inner.supply.utxo_count,
            SupplyState::from(&addr).utxo_count
        );
        assert_eq!(cohort.inner.supply.value, SupplyState::from(&addr).value);
        assert_eq!(cohort.inner.realized.cap(), Cents::new(17_500));
        assert_eq!(cohort.inner.realized.profit(), Cents::new(1_250));
        assert_eq!(cohort.inner.sent, Sats::new(25_000_000));
        assert_eq!(cohort.inner.realized.value_destroyed(), Cents::ZERO);

        cohort.subtract(&addr);
        assert_eq!(cohort.addr_count, 0);
        assert_eq!(cohort.inner.supply.utxo_count, 0);
        assert_eq!(cohort.inner.supply.value, Sats::ZERO);
        assert_eq!(cohort.inner.realized.cap(), Cents::ZERO);
    }
}
