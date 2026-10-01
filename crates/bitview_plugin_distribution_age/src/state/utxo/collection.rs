use crate::compute::ComputeContext;
use bitview_cohort::{AgeRange, AgeRangeId, ByEpoch, Class, Term};
use brk_error::Result;
use brk_types::{Age, CostBasisSnapshot, Height};
use rayon::scope as RayonScope;
use statedb::Amount;

use super::{CostBasisFenwick, UTXOCohortState, UTXOTransientState};
use crate::state::{CoreRealizedState, RealizedState, WithCapital, WithoutCapital, supply};

pub struct UTXOStates {
    pub age_range: AgeRange<UTXOCohortState<RealizedState, WithCapital>>,
    pub epoch: ByEpoch<UTXOCohortState<CoreRealizedState, WithoutCapital>>,
    pub class: Class<UTXOCohortState<CoreRealizedState, WithoutCapital>>,
    pub transient: UTXOTransientState,
}

impl UTXOStates {
    pub fn new() -> Self {
        Self {
            age_range: AgeRange::new(|_| UTXOCohortState::new()),
            epoch: ByEpoch::new(|_| UTXOCohortState::new()),
            class: Class::new(|_| UTXOCohortState::new()),
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

    pub fn init_fenwick_if_needed(&mut self) {
        if self.transient.fenwick.is_initialized() {
            return;
        }

        let Self {
            age_range,
            transient,
            ..
        } = self;
        transient.fenwick.compute_is_sth();
        let maps = AgeRangeId::ALL.iter().filter_map(|&id| {
            let map = id.select(age_range).cost_basis_map();
            (!map.is_empty()).then(|| (map, id.term() == Term::Sth))
        });
        transient.fenwick.bulk_init(maps);
    }

    pub fn update_fenwick_from_pending(&mut self) {
        if !self.transient.fenwick.is_initialized() {
            return;
        }

        let Self {
            age_range,
            transient,
            ..
        } = self;
        for &id in AgeRangeId::ALL {
            let is_sth = transient.fenwick.is_sth(id);
            id.select(age_range)
                .for_each_cost_basis_pending(|&price, delta| {
                    transient.fenwick.apply_delta(price, delta, is_sth);
                });
        }
    }

    pub fn fenwick(&self) -> &CostBasisFenwick {
        &self.transient.fenwick
    }
}
