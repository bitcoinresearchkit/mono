use bitview_cohort::{Age, AgeRange, ByEpoch, Class};
use bitview_primitives::CostBasisSnapshot;
use brk_types::Height;
use rayon::scope as RayonScope;
use statedb::Amount;

use super::{UTXOCohortState, UTXOTransientState};
use crate::{
    compute::ComputeContext,
    state::{CoreRealizedState, RealizedState, supply},
};

pub struct UTXOStates {
    pub age_range: AgeRange<UTXOCohortState<RealizedState>>,
    pub epoch: ByEpoch<UTXOCohortState<CoreRealizedState>>,
    pub class: Class<UTXOCohortState<CoreRealizedState>>,
    pub transient: UTXOTransientState,
}

impl UTXOStates {
    pub fn new() -> Self {
        Self {
            age_range: AgeRange::from_fn(|_| UTXOCohortState::default()),
            epoch: ByEpoch::from_fn(|_| UTXOCohortState::default()),
            class: Class::from_fn(|_| UTXOCohortState::default()),
            transient: UTXOTransientState::default(),
        }
    }

    /// Restores freshly constructed states from the origin balances. Restoring moves no
    /// block activity, so the states stay ready for the next block.
    pub fn restore_origins(&mut self, amounts: &[Amount], ctx: &ComputeContext<'_>) {
        let Some(last) = amounts.len().checked_sub(1) else {
            return;
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
