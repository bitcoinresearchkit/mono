use bitview_cohort::{Age, AgeRange, ByHalvingEpoch, Vintage};
use bitview_distribution::walk::Origins;
use bitview_primitives::CostBasisSnapshot;
use brk_types::Height;
use rayon::scope as RayonScope;

use super::UTXOCohortState;
use crate::state::{CoreRealizedState, RealizedState, supply};

pub struct UTXOStates {
    pub age_range: AgeRange<UTXOCohortState<RealizedState>>,
    pub halving_epoch: ByHalvingEpoch<UTXOCohortState<CoreRealizedState>>,
    pub vintage: Vintage<UTXOCohortState<CoreRealizedState>>,
}

impl UTXOStates {
    pub fn new() -> Self {
        Self {
            age_range: AgeRange::from_fn(|_| UTXOCohortState::default()),
            halving_epoch: ByHalvingEpoch::from_fn(|_| UTXOCohortState::default()),
            vintage: Vintage::from_fn(|_| UTXOCohortState::default()),
        }
    }

    /// Restores freshly constructed states from the origin balances. Restoring moves no
    /// block activity, so the states stay ready for the next block.
    pub fn restore_origins(&mut self, origins: &Origins<'_>) {
        let Some(last) = origins.amounts.len().checked_sub(1) else {
            return;
        };
        let timestamp = origins.timestamps[last];
        for (h, &amount) in origins.amounts.iter().enumerate() {
            let origin_timestamp = origins.timestamps[h];
            let snapshot = CostBasisSnapshot::from_utxo(origins.prices[h], &supply(amount));
            self.age_range
                .get_mut(Age::new(timestamp, origin_timestamp))
                .increment_snapshot(&snapshot);
            if let Some(state) = self.halving_epoch.get_mut(Height::from(h)) {
                state.increment_snapshot(&snapshot);
            }
            if let Some(state) = self.vintage.get_mut(origin_timestamp) {
                state.increment_snapshot(&snapshot);
            }
        }
        RayonScope(|scope| {
            for state in self.age_range.iter_mut() {
                scope.spawn(move |_| state.finish_restore());
            }
            for state in self.halving_epoch.iter_mut() {
                scope.spawn(move |_| state.finish_restore());
            }
            for state in self.vintage.iter_mut() {
                scope.spawn(move |_| state.finish_restore());
            }
        });
    }

    pub fn apply_pending(&mut self) {
        let Self {
            age_range,
            halving_epoch,
            vintage,
            ..
        } = self;
        RayonScope(|scope| {
            for state in age_range.iter_mut() {
                scope.spawn(move |_| state.apply_pending());
            }
            for state in halving_epoch.iter_mut() {
                scope.spawn(move |_| state.apply_pending());
            }
            for state in vintage.iter_mut() {
                scope.spawn(move |_| state.apply_pending());
            }
        });
    }

    pub fn reset_block(&mut self) {
        self.age_range
            .iter_mut()
            .for_each(|state| state.reset_block());
        self.halving_epoch
            .iter_mut()
            .for_each(|state| state.reset_block());
        self.vintage
            .iter_mut()
            .for_each(|state| state.reset_block());
    }
}
