use bitview_cohort::{AmountRange, AmountRangeId};
use bitview_primitives::Count;
use brk_types::{Cents, Height};

use super::AddrCohortState;
use crate::{addr::FundedAddrCountsVecs, metrics::BalanceMetrics};

pub struct AddrStates {
    pub amount_range: AmountRange<AddrCohortState>,
}

impl AddrStates {
    pub fn new() -> Self {
        Self {
            amount_range: AmountRange::new(|_| AddrCohortState::new()),
        }
    }

    pub fn restore(
        &mut self,
        metrics: &BalanceMetrics,
        funded: &FundedAddrCountsVecs,
        height: Height,
    ) -> Option<()> {
        let previous_height = height.decremented()?;

        let supply = metrics.supply_source.checkpoint(previous_height)?;
        let output_count = metrics.utxo_count.checkpoint(previous_height)?;
        let addr_count = funded.balance.checkpoint(previous_height)?;

        for amount in AmountRangeId::ALL {
            let state = amount.select_mut(&mut self.amount_range);
            state.inner.supply.value = *amount.select(&supply);
            state.inner.supply.utxo_count = u64::from(*amount.select(&output_count));
            state.addr_count = u64::from(*amount.select(&addr_count));
        }

        Some(())
    }

    pub fn push(
        &self,
        metrics: &mut BalanceMetrics,
        funded: &mut FundedAddrCountsVecs,
        height_price: Cents,
    ) {
        metrics.push(&self.amount_range, height_price);
        funded.push_balance(AmountRange::from_fn(|amount| {
            Count::from(amount.select(&self.amount_range).addr_count)
        }));
    }

    pub fn reset_block(&mut self) {
        self.amount_range
            .iter_mut()
            .for_each(|state| state.inner.reset_single_iteration_values());
    }
}
