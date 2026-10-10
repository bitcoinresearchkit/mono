use brk_types::Cents;
use derive_more::{Deref, DerefMut};

use super::UTXOCohortState as CommonCohort;
use crate::state::{CostBasisData, RealizedOps, UnrealizedState};

#[derive(Deref, DerefMut)]
pub struct MappedUTXOCohortState<R: RealizedOps>(pub CommonCohort<R, CostBasisData>);

impl<R: RealizedOps> Default for MappedUTXOCohortState<R> {
    fn default() -> Self {
        Self(CommonCohort::new(CostBasisData::default()))
    }
}

impl<R: RealizedOps> MappedUTXOCohortState<R> {
    pub fn finish_restore(&mut self) {
        self.cost_basis.finish_restore();
    }

    pub fn compute_unrealized_state(&mut self, height_price: Cents) -> UnrealizedState {
        self.cost_basis.compute_unrealized_state(height_price)
    }
}
