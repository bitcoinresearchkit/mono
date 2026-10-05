use bitview_primitives::{FundedAddrData, SupplyState};
use brk_error::Result;
use brk_types::{Cents, Sats};

use super::{super::CohortState, super::MinimalRealizedState};

/// Mutable state for one address balance cohort.
pub struct AddrCohortState {
    pub addr_count: u64,
    pub inner: CohortState<MinimalRealizedState, ()>,
}

impl AddrCohortState {
    pub fn new() -> Self {
        Self {
            addr_count: 0,
            inner: CohortState::new(()),
        }
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
            .increment_addr(&supply, addr_data.realized_cap_raw());
    }

    pub fn subtract(&mut self, addr_data: &FundedAddrData) {
        let supply = SupplyState::from(addr_data);
        debug_assert!(
            self.addr_count > 0
                && self.inner.supply.utxo_count >= supply.utxo_count
                && self.inner.supply.value >= supply.value,
            "address not tracked in this cohort: addr_count={}, supply={}, addr={addr_data}",
            self.addr_count,
            self.inner.supply
        );
        self.addr_count -= 1;
        self.inner
            .decrement_addr(&supply, addr_data.realized_cap_raw());
    }
}
