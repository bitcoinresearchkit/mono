use crate::state::{CostBasisData, PendingDelta, UnrealizedState, cost_basis::Accumulate};
use bitview_plugin_distribution_common::state::{RealizedOps, UTXOCohortState as CommonCohort};
use brk_types::{Cents, CentsCompact, Sats};
use derive_more::{Deref, DerefMut};
use std::collections::BTreeMap;

#[derive(Deref, DerefMut)]
pub struct UTXOCohortState<R: RealizedOps, S: Accumulate>(pub CommonCohort<R, CostBasisData<S>>);

impl<R: RealizedOps, S: Accumulate> UTXOCohortState<R, S> {
    pub fn new() -> Self {
        Self(CommonCohort::new(CostBasisData::default()))
    }
    pub fn finish_restore(&mut self) {
        self.cost_basis.finish_restore();
    }

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
