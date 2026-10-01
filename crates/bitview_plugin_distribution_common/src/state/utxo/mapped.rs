use std::collections::BTreeMap;

use crate::state::{Accumulate, CostBasisData, RealizedOps, UnrealizedState};
use brk_types::{Cents, CentsCompact, Sats};
use derive_more::{Deref, DerefMut};

use super::UTXOCohortState as CommonCohort;

#[derive(Deref, DerefMut)]
pub struct MappedUTXOCohortState<R: RealizedOps, S: Accumulate>(
    pub CommonCohort<R, CostBasisData<S>>,
);

impl<R: RealizedOps, S: Accumulate> MappedUTXOCohortState<R, S> {
    pub fn new() -> Self {
        Self(CommonCohort::new(CostBasisData::default()))
    }
    pub fn finish_restore(&mut self) {
        self.cost_basis.finish_restore();
    }

    pub fn compute_unrealized_state(&mut self, height_price: Cents) -> UnrealizedState {
        self.cost_basis.compute_unrealized_state(height_price)
    }

    pub fn cost_basis_map(&self) -> &BTreeMap<CentsCompact, Sats> {
        self.cost_basis.map()
    }
}
