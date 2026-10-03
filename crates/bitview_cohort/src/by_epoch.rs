use bitview_primitives::Halving;
#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use brk_types::Height;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CohortId, CohortName};

/// Epoch names
pub const EPOCH_NAMES: ByEpoch<CohortName> = ByEpoch {
    _0: CohortName::new("epoch_0", "0", "Epoch 0"),
    _1: CohortName::new("epoch_1", "1", "Epoch 1"),
    _2: CohortName::new("epoch_2", "2", "Epoch 2"),
    _3: CohortName::new("epoch_3", "3", "Epoch 3"),
    _4: CohortName::new("epoch_4", "4", "Epoch 4"),
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct ByEpoch<T> {
    /// Uses UTXOs created during subsidy-halving epoch 0.
    _0: T,
    /// Uses UTXOs created during subsidy-halving epoch 1.
    _1: T,
    /// Uses UTXOs created during subsidy-halving epoch 2.
    _2: T,
    /// Uses UTXOs created during subsidy-halving epoch 3.
    _3: T,
    /// Uses UTXOs created during subsidy-halving epoch 4.
    _4: T,
}

define_cohort_id!(
    EpochId for ByEpoch {
        _0 => _0,
        _1 => _1,
        _2 => _2,
        _3 => _3,
        _4 => _4,
    }
);

impl<T> ByEpoch<T> {
    pub fn new(mut create: impl FnMut(CohortId) -> T) -> Self {
        Self::from_fn(|id| create(id.cohort()))
    }

    pub(crate) fn try_new<E>(mut create: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self, E> {
        Self::try_from_fn(|id| create(id.cohort()))
    }

    pub fn mut_vec_from_height(&mut self, height: Height) -> Option<&mut T> {
        let epoch = Halving::from(height);
        if epoch == Halving::new(0) {
            Some(&mut self._0)
        } else if epoch == Halving::new(1) {
            Some(&mut self._1)
        } else if epoch == Halving::new(2) {
            Some(&mut self._2)
        } else if epoch == Halving::new(3) {
            Some(&mut self._3)
        } else if epoch == Halving::new(4) {
            Some(&mut self._4)
        } else {
            None
        }
    }
}

impl EpochId {
    pub(crate) const fn cohort(self) -> CohortId {
        CohortId::Epoch(self)
    }
}
