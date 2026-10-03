use bitview_primitives::Lengths;
use parking_lot::{ArcRwLockReadGuard, RawRwLock};

/// Pins a published immutable index prefix without waiting for append/compute.
/// Rollback must lower the shared lengths before changing that prefix, so it
/// waits for this guard. Mutable aggregates are not protected by these bounds.
pub struct SafeLengths {
    _guard: ArcRwLockReadGuard<RawRwLock, ()>,
    lengths: Lengths,
}

impl SafeLengths {
    pub(crate) fn new(guard: ArcRwLockReadGuard<RawRwLock, ()>, lengths: Lengths) -> Self {
        Self {
            _guard: guard,
            lengths,
        }
    }

    /// Copy the bounds, not their protection. Retain this guard until every
    /// read authorized by these lengths has completed.
    pub fn lengths(&self) -> Lengths {
        self.lengths
    }
}
