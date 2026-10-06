use bitview_cohort::{AmountRange, AmountRangeId};
use bitview_primitives::{FundedAddrData, SupplyState};
use brk_error::Result;
use brk_types::{Cents, CentsSats, Sats};

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

    fn apply(&mut self, change: Change) {
        match change {
            Change::Add { supply, cap } => {
                self.addr_count += 1;
                self.inner.increment_addr(&supply, cap);
            }
            Change::Subtract { supply, cap } => {
                debug_assert!(
                    self.addr_count > 0
                        && self.inner.supply.utxo_count >= supply.utxo_count
                        && self.inner.supply.value >= supply.value,
                    "address not tracked in this cohort: addr_count={}, supply={}, removed={supply}",
                    self.addr_count,
                    self.inner.supply
                );
                self.addr_count -= 1;
                self.inner.decrement_addr(&supply, cap);
            }
            Change::Receive { supply, cap } => self.inner.increment_addr(&supply, cap),
            Change::Send {
                value,
                current_price,
                prev_ps,
            } => self.inner.send_addr(
                &SupplyState {
                    utxo_count: 1,
                    value,
                },
                current_price,
                prev_ps,
            ),
        }
    }
}

#[derive(Clone, Copy)]
enum Change {
    Add {
        supply: SupplyState,
        cap: CentsSats,
    },
    Subtract {
        supply: SupplyState,
        cap: CentsSats,
    },
    Receive {
        supply: SupplyState,
        cap: CentsSats,
    },
    Send {
        value: Sats,
        current_price: Cents,
        prev_ps: CentsSats,
    },
}

/// One shard's cohort changes in one block, in order. Shards are processed apart and their logs
/// applied to the shared cohorts afterwards: cohort state only sums, and each address's own
/// changes keep their order.
#[derive(Default)]
pub struct CohortLog(Vec<(AmountRangeId, Change)>);

impl CohortLog {
    pub fn add(&mut self, bucket: AmountRangeId, addr_data: &FundedAddrData) {
        self.0.push((
            bucket,
            Change::Add {
                supply: SupplyState::from(addr_data),
                cap: addr_data.realized_cap_raw(),
            },
        ));
    }

    pub fn subtract(&mut self, bucket: AmountRangeId, addr_data: &FundedAddrData) {
        self.0.push((
            bucket,
            Change::Subtract {
                supply: SupplyState::from(addr_data),
                cap: addr_data.realized_cap_raw(),
            },
        ));
    }

    pub fn receive_outputs(
        &mut self,
        bucket: AmountRangeId,
        addr_data: &mut FundedAddrData,
        value: Sats,
        price: Cents,
        output_count: u32,
    ) {
        let cap = addr_data.receive_outputs(value, price, output_count);
        self.0.push((
            bucket,
            Change::Receive {
                supply: SupplyState {
                    utxo_count: u64::from(output_count),
                    value,
                },
                cap,
            },
        ));
    }

    pub fn send(
        &mut self,
        bucket: AmountRangeId,
        addr_data: &mut FundedAddrData,
        value: Sats,
        current_price: Cents,
        prev_price: Cents,
    ) -> Result<()> {
        let prev_ps = addr_data.send(value, prev_price)?;
        self.0.push((
            bucket,
            Change::Send {
                value,
                current_price,
                prev_ps,
            },
        ));
        Ok(())
    }

    pub fn apply_to(&self, cohorts: &mut AmountRange<AddrCohortState>) {
        for &(bucket, change) in &self.0 {
            bucket.select_mut(cohorts).apply(change);
        }
    }
}
