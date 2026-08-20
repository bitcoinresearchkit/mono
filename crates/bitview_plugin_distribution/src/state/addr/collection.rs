use brk_error::Result;

use std::path::Path;

use bitview_cohort::{AmountRange, AmountRangeId, CohortContext, Filter};
use brk_types::{Cents, Height, StoredU64};
use rayon::prelude::*;
use vecdb::{ColumnId, ReadableVec};

use crate::{addr::FundedAddrCountsVecs, metrics::CohortMetrics};

use super::AddrCohortState;

pub struct AddrStates {
    pub amount_range: AmountRange<AddrCohortState>,
    starting_height: Height,
}

impl AddrStates {
    pub fn new(path: &Path) -> Self {
        Self {
            amount_range: AmountRange::new(|filter: Filter, name| {
                let name = CohortContext::Addr.full_name(&filter, name);
                AddrCohortState::new(path, &name)
            }),
            starting_height: Height::ZERO,
        }
    }

    pub fn import(
        &mut self,
        metrics: &CohortMetrics,
        funded: &FundedAddrCountsVecs,
        height: Height,
    ) -> Result<bool> {
        let Some(previous_height) = height.decremented() else {
            self.starting_height = Height::ZERO;
            return Ok(true);
        };

        for state in self.amount_range.iter_mut() {
            let imported_height = state.inner.import_at_or_before(previous_height)?;
            if imported_height != previous_height {
                return Ok(false);
            }
        }

        let Some(supply) = metrics
            .supply
            .total
            .addr_balance
            .matrix
            .collect_one(previous_height)
        else {
            return Ok(false);
        };
        let Some(output_count) = metrics
            .outputs
            .unspent_count
            .addr_balance
            .matrix
            .collect_one(previous_height)
        else {
            return Ok(false);
        };
        let Some(addr_count) = funded.balance.matrix.collect_one(previous_height) else {
            return Ok(false);
        };

        for amount in AmountRangeId::ALL {
            let state = amount.select_mut(&mut self.amount_range);
            state.inner.supply.value = *amount.select(&supply);
            state.inner.supply.utxo_count = u64::from(*amount.select(&output_count));
            state.addr_count = u64::from(*amount.select(&addr_count));
            state.inner.restore_realized_cap();
        }

        self.starting_height = previous_height.incremented();
        Ok(self.starting_height == height)
    }

    pub fn reset(&mut self) -> Result<()> {
        self.starting_height = Height::ZERO;
        for state in self.amount_range.iter_mut() {
            state.reset();
            state.inner.reset_cost_basis_data_if_needed()?;
        }
        Ok(())
    }

    pub fn push(
        &self,
        metrics: &mut CohortMetrics,
        funded: &mut FundedAddrCountsVecs,
        height: Height,
        height_price: Cents,
    ) {
        if height < self.starting_height {
            return;
        }
        metrics.push_addr_balance(&self.amount_range, height_price);
        funded.push_balance(AmountRange::from_fn(|amount| {
            StoredU64::from(amount.select(&self.amount_range).addr_count)
        }));
    }

    pub fn reset_block(&mut self) {
        self.amount_range
            .iter_mut()
            .for_each(|state| state.inner.reset_single_iteration_values());
    }

    pub fn write(&mut self, height: Height, cleanup: bool) -> Result<()> {
        self.amount_range
            .par_iter_mut()
            .try_for_each(|state| state.inner.write(height, cleanup))
    }
}
