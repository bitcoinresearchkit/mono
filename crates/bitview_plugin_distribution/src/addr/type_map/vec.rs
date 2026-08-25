use bitview_cohort::ByAddrType;
use derive_more::{Deref, DerefMut};

/// A vector for each address type.
#[derive(Debug, Deref, DerefMut)]
pub struct AddrTypeToVec<T>(ByAddrType<Vec<T>>);

impl<T> Default for AddrTypeToVec<T> {
    fn default() -> Self {
        Self(ByAddrType::default())
    }
}

impl<T> AddrTypeToVec<T> {
    /// Create with pre-allocated capacity per address type.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(ByAddrType::from_fn(|_| Vec::with_capacity(capacity)))
    }

    pub fn with_capacities(capacities: ByAddrType<usize>) -> Self {
        Self(ByAddrType::from_fn(|id| {
            Vec::with_capacity(*capacities.get_unwrap(id.output_type()))
        }))
    }

    /// Consume the wrapper and return its vectors.
    pub fn into_inner(self) -> ByAddrType<Vec<T>> {
        self.0
    }
}
