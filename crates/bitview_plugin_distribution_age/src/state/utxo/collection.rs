use bitview_cohort::{Age, AgeRange, ByEpoch, Class};
use bitview_primitives::CostBasisSnapshot;
use brk_error::Result;
use brk_types::Height;
use rayon::scope as RayonScope;
use statedb::Amount;

use super::{UTXOCohortState, UTXOTransientState};
use crate::{
    compute::ComputeContext,
    state::{CoreRealizedState, RealizedState, WithCapital, WithoutCapital, supply},
};

pub struct UTXOStates {
    pub age_range: AgeRange<UTXOCohortState<RealizedState, WithCapital>>,
    pub epoch: ByEpoch<UTXOCohortState<CoreRealizedState, WithoutCapital>>,
    pub class: Class<UTXOCohortState<CoreRealizedState, WithoutCapital>>,
    pub transient: UTXOTransientState,
}

impl UTXOStates {
    pub fn new() -> Self {
        Self {
            age_range: AgeRange::new(|_| UTXOCohortState::default()),
            epoch: ByEpoch::new(|_| UTXOCohortState::default()),
            class: Class::new(|_| UTXOCohortState::default()),
            transient: UTXOTransientState::default(),
        }
    }

    pub fn reset(&mut self) -> Result<()> {
        for state in self.age_range.iter_mut() {
            state.reset();
            state.init_cost_basis();
        }
        for state in self.epoch.iter_mut() {
            state.reset();
            state.init_cost_basis();
        }
        for state in self.class.iter_mut() {
            state.reset();
            state.init_cost_basis();
        }
        self.transient = UTXOTransientState::default();
        Ok(())
    }

    pub fn restore_origins(&mut self, amounts: &[Amount], ctx: &ComputeContext<'_>) -> Result<()> {
        self.reset()?;
        let Some(last) = amounts.len().checked_sub(1) else {
            return Ok(());
        };
        let timestamp = ctx.height_to_timestamp[last];
        for (h, &amount) in amounts.iter().enumerate() {
            let origin_timestamp = ctx.height_to_timestamp[h];
            let snapshot = CostBasisSnapshot::from_utxo(ctx.height_to_price[h], &supply(amount));
            self.age_range
                .get_mut(Age::new(timestamp, origin_timestamp))
                .increment_snapshot(&snapshot);
            if let Some(state) = self.epoch.mut_vec_from_height(Height::from(h)) {
                state.increment_snapshot(&snapshot);
            }
            if let Some(state) = self.class.mut_vec_from_timestamp(origin_timestamp) {
                state.increment_snapshot(&snapshot);
            }
        }
        RayonScope(|scope| {
            for state in self.age_range.iter_mut() {
                scope.spawn(move |_| state.finish_restore());
            }
            for state in self.epoch.iter_mut() {
                scope.spawn(move |_| state.finish_restore());
            }
            for state in self.class.iter_mut() {
                scope.spawn(move |_| state.finish_restore());
            }
        });
        self.reset_block();
        Ok(())
    }

    pub fn apply_pending(&mut self) {
        let Self {
            age_range,
            epoch,
            class,
            ..
        } = self;
        RayonScope(|scope| {
            for state in age_range.iter_mut() {
                scope.spawn(move |_| state.apply_pending());
            }
            for state in epoch.iter_mut() {
                scope.spawn(move |_| state.apply_pending());
            }
            for state in class.iter_mut() {
                scope.spawn(move |_| state.apply_pending());
            }
        });
    }

    pub fn reset_block(&mut self) {
        self.age_range
            .iter_mut()
            .for_each(|state| state.reset_single_iteration_values());
        self.epoch
            .iter_mut()
            .for_each(|state| state.reset_single_iteration_values());
        self.class
            .iter_mut()
            .for_each(|state| state.reset_single_iteration_values());
    }
}
